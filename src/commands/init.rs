// src/commands/init.rs

use crate::{db, InitArgs};
use anyhow::Result;
use std::path::PathBuf;

pub fn handle_init(_args: &InitArgs, db_path: &PathBuf) -> Result<()> {
    // _args 暂时未使用，但为未来的 `upgrade` 功能保留

    if db_path.exists() {
        println!("数据库已存在于: {}", db_path.display());
        println!("无需执行初始化操作。");
    } else {
        println!("数据库不存在，正在创建并初始化...");
        match db::initialize_db(db_path) {
            Ok(_) => {
                println!("✅ 数据库成功初始化于: {}", db_path.display());
                println!("   - 已创建 'logs', 'backup', 'configs' 表。");
                println!(
                    "   - 已为 'backup' 表设置自动清理触发器 (上限 {} 条)。",
                    db::MAX_BACKUP_ENTRIES
                ); // 这里的100可以从db模块导入常量
            }
            Err(e) => {
                // 将 rusqlite::Error 转换为 anyhow::Error 并返回
                return Err(e.into());
            }
        }
    }

    Ok(())
}
