// Copyright 2025 ABdeeglr Ramsay
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// src/db.rs

use chrono::Local;
use rusqlite::ffi::ErrorCode;
use rusqlite::{Connection, Result, Transaction};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// 在开始迁移前，为数据库文件创建一个带时间戳的备份。
///
/// # Arguments
/// * `db_path` - 原始数据库文件的路径。
///
/// # Returns
/// 成功时返回备份文件的路径 `Result<PathBuf>`，失败时返回 IO 错误。
fn backup_database(db_path: &Path) -> std::io::Result<PathBuf> {
    // 获取父目录，如果不存在则默认为当前目录 "."
    let parent_dir = db_path.parent().unwrap_or_else(|| Path::new("."));

    // 从原始路径中解析出文件名和扩展名
    let file_stem = db_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("database");
    let extension = db_path.extension().and_then(|s| s.to_str()).unwrap_or("db");

    // 生成带 YYYY-MM-DD_HHMMSS 格式的时间戳，确保唯一性
    let timestamp = Local::now().format("%Y-%m-%d_%H%M%S").to_string();
    let backup_file_name = format!("{}_bak_{}.{}", file_stem, timestamp, extension);
    let backup_path = parent_dir.join(backup_file_name);

    // 执行文件复制
    fs::copy(db_path, &backup_path)?;

    Ok(backup_path)
}

// 定义备份表的最大条目数，超过此数量将自动清理
pub const MAX_BACKUP_ENTRIES: usize = 100;

// 定义程序所使用的数据库版本（并不是程序版本），用于版本升级时对数据库进行操作
pub const DLOG_DB_VERSION: u32 = 2;

/**
 * 检查 dlog 所使用的数据库模式的版本。
 */
fn get_db_version(conn: &Connection) -> Result<u32> {
    // 检查 configs 表是否存在，如果不存在，说明是 v1 或更早版本
    let table_exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='configs')",
        [],
        |row| row.get(0),
    )?;

    if !table_exists {
        // 在 v1 中，configs 表不存在，因此版本可以认为是 1 (或任何低于2的数字)
        return Ok(1);
    }

    // 如果 configs 表存在，查询版本号
    let version_str: Result<String> = conn.query_row(
        "SELECT value FROM configs WHERE key = 'db_version'",
        [],
        |row| row.get(0),
    );

    match version_str {
        Ok(s) => Ok(s.parse().unwrap_or(1)), // 如果解析失败，也视为旧版本
        Err(_) => Ok(1),                     // 如果没有 db_version 记录，也视为旧版本
    }
}

/// 执行从数据库版本 1 到 2 的迁移。
///
/// 此函数在一个现有的事务中运行，以确保原子性。
/// 它会：
/// 1. 创建 `backup` 表。
/// 2. 创建 `configs` 表。
/// 3. 在 `configs` 表中将 `db_version` 设置为 2
fn migrate_to_v2(tx: &Transaction) -> Result<()> {
    println!("  -> Migrating to version 2...");

    // 步骤 1: 创建 backup 表
    tx.execute(
        "CREATE TABLE IF NOT EXISTS backup (
            id          INTEGER PRIMARY KEY,
            timestamp   TEXT NOT NULL,
            directory   TEXT NOT NULL,
            content     TEXT NOT NULL,
            tags        TEXT,
            metadata    TEXT,
            level       TEXT
        )",
        (),
    )?;
    println!("     - Table 'backup' created.");

    // 步骤 2: 创建 configs 表
    tx.execute(
        "CREATE TABLE IF NOT EXISTS configs (
            key     TEXT PRIMARY KEY,
            value   TEXT NOT NULL
        )",
        (),
    )?;
    println!("     - Table 'configs' created.");

    // 步骤 3: 插入或更新数据库版本号为 2
    tx.execute(
        "INSERT OR REPLACE INTO configs (key, value) VALUES ('db_version', ?1)",
        [&DLOG_DB_VERSION.to_string()],
    )?;
    println!("     - Database version set to {}.", DLOG_DB_VERSION);

    // 4. 创建触发器，用于在 backup 表条目过多时自动清理
    let trigger_sql = format!(
        "CREATE TRIGGER IF NOT EXISTS trim_backup_logs
         AFTER INSERT ON backup
         WHEN (SELECT COUNT(*) FROM backup) > {limit}
         BEGIN
             DELETE FROM backup WHERE id IN (
                 SELECT id FROM backup ORDER BY timestamp ASC LIMIT (SELECT COUNT(*) - {limit} FROM backup)
             );
         END;",
        limit = MAX_BACKUP_ENTRIES
    );
    tx.execute(&trigger_sql, ())?;

    Ok(())
}

/// 数据库版本更新的主函数
pub fn run_migrations(db_path: &PathBuf) -> Result<()> {
    if !db_path.exists() {
        return Ok(());
    }

    let mut conn = Connection::open(db_path)?;
    let current_db_version = get_db_version(&conn)?;

    if current_db_version < DLOG_DB_VERSION {
        // --- 备份逻辑 ---
        println!("Database requires migration. Creating a backup before proceeding...");
        match backup_database(db_path) {
            Ok(backup_path) => {
                println!(
                    "✅ Backup created successfully at: {}",
                    backup_path.display()
                );
            }
            Err(e) => {
                // 如果备份失败，立即中止并返回错误
                eprintln!(
                    "🔥 Critical: Failed to create database backup. Migration aborted. Error: {}",
                    e
                );
                // 将 io::Error 转换为 rusqlite::Error 以便上层统一处理
                return Err(rusqlite::Error::ToSqlConversionFailure(e.into()));
            }
        };
        // --- 备份逻辑结束 ---

        println!("An older version of the database (version {}) has been detected, requiring an upgrade to version {}...",
            current_db_version,
            DLOG_DB_VERSION
        );

        // 使用事务来确保升级的完整性
        let tx = conn.transaction()?;

        // 渐进式版本更新操作
        if current_db_version < 2 {
            // 将事务传递给特定的迁移函数
            migrate_to_v2(&tx)?;
        }
        // 如果未來有 v3, 在這裡加:
        // if current_db_version < 3 { migrate_to_v3(&tx)?; ... }

        tx.commit()?;
        println!("🎉 数据库升级成功！");
    }

    Ok(())
}

pub fn initialize_db(db_path: &Path) -> Result<()> {
    // 检查并创建父目录
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error {
                        code: ErrorCode::CannotOpen,
                        extended_code: 0,
                    },
                    Some(format!("Failed to create directory: {}", e)),
                )
            })?;
        }
    }

    let conn = Connection::open(db_path)?;

    // 1. 创建核心的 logs 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id          INTEGER PRIMARY KEY,
            timestamp   TEXT NOT NULL,
            directory   TEXT NOT NULL,
            content     TEXT NOT NULL,
            tags        TEXT,
            metadata    TEXT, -- 用于存储哈希标识符
            level       TEXT
        )",
        (),
    )?;

    // 2. 创建结构相同的 backup 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS backup (
            id          INTEGER PRIMARY KEY,
            timestamp   TEXT NOT NULL,
            directory   TEXT NOT NULL,
            content     TEXT NOT NULL,
            tags        TEXT,
            metadata    TEXT,
            level       TEXT
        )",
        (),
    )?;

    // 3. 创建 configs 表，用于存储键值对配置
    conn.execute(
        "CREATE TABLE IF NOT EXISTS configs (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        (),
    )?;

    // 4. 创建触发器，用于在 backup 表条目过多时自动清理
    let trigger_sql = format!(
        "CREATE TRIGGER IF NOT EXISTS trim_backup_logs
         AFTER INSERT ON backup
         WHEN (SELECT COUNT(*) FROM backup) > {limit}
         BEGIN
             DELETE FROM backup WHERE id IN (
                 SELECT id FROM backup ORDER BY timestamp ASC LIMIT (SELECT COUNT(*) - {limit} FROM backup)
             );
         END;",
        limit = MAX_BACKUP_ENTRIES
    );
    conn.execute(&trigger_sql, ())?;

    Ok(())
}
