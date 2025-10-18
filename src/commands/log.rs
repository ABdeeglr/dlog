// src/commands/log.rs

use crate::LogArgs;
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::Connection;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead};
use std::path::PathBuf;

/// 计算给定字符串的 u64 哈希值。
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn handle_log(args: &LogArgs, db_path: &PathBuf) -> Result<()> {
    // 1. 获取日志内容
    let content = if let Some(message) = &args.message {
        // 如果用户通过 -m 提供了短消息，直接使用
        message.clone()
    } else {
        // 否则，进入交互式输入模式以获取长消息
        println!("请输入日志内容 (按 Ctrl+D 结束):");
        let mut lines = io::stdin().lock().lines();
        let mut input_content = String::new();
        while let Some(line) = lines.next() {
            let line = line.context("无法从标准输入读取行")?;
            input_content.push_str(&line);
            input_content.push('\n');
        }
        // 移除最后一个多余的换行符
        input_content.trim_end().to_string()
    };

    // 如果内容为空，则不记录
    if content.is_empty() {
        println!("日志内容为空，已取消操作。");
        return Ok(());
    }

    // 2. 准备要存入数据库的数据
    let timestamp = Utc::now().to_rfc3339();
    let directory = if args.global {
        "global".to_string()
    } else {
        std::env::current_dir()?
            .to_str()
            .context("无法将当前目录转换为字符串")?
            .to_string()
    };

    // 3. 生成并存储哈希标识符
    let hash_input = format!("{}{}", timestamp, content);
    let hash = calculate_hash(&hash_input);
    let metadata = format!("{:x}", hash); // 格式化为十六进制字符串

    let tags = args.tags.clone().unwrap_or_default();

    // 4. 连接数据库并插入数据
    let conn =
        Connection::open(db_path).context(format!("无法连接到数据库: {}", db_path.display()))?;

    conn.execute(
        "INSERT INTO logs (timestamp, directory, content, tags, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![timestamp, directory, content, tags, metadata],
    )?;

    // 5. 向用户提供反馈
    let short_hash = &metadata[..7]; // 取哈希的前7位作为短哈希，更像git
    println!("✅ 日志已成功记录！");
    println!("   唯一标识: {}", short_hash);

    Ok(())
}
