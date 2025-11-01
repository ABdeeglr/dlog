// src/commands/get.rs

use crate::GetArgs;
use anyhow::{Context, Result as AnyhowResult};
use rusqlite::{params_from_iter, Connection, Result, Row, ToSql};
use std::path::PathBuf;
use chrono::{DateTime, FixedOffset, ParseError, Local};
use serde::Serialize;
use std::fmt;

// 定义一个 LogEntry 结构体，用于在最后阶段存放和显示日志数据
#[derive(Debug, Clone, Serialize)]
struct Log {
    id: i32,
    timestamp: String,
    directory: String,
    content: String,
    tags: Option<String>,
    metadata: Option<String>,
    level: Option<String>,
}

impl Log {
    /// 从数据库行转换为 Log 结构体
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Log {
            id: row.get("id")?,
            timestamp: row.get("timestamp")?,
            directory: row.get("directory")?,
            content: row.get("content")?,
            tags: row.get("tags")?,
            metadata: row.get("metadata")?,
            level: row.get("level")?,
        })
    }

    /// 将 UTC 时间转换为本地时间字符串
    fn get_local_time(&self) -> Option<String> {
        // 复用你现有的时间解析逻辑
        let parse_result = DateTime::parse_from_rfc3339(&self.timestamp)
            .or_else(|_| DateTime::parse_from_str(&self.timestamp, "%+"))
            .or_else(|_| manual_time_parse(&self.timestamp));

        parse_result.ok().map(|utc_time| {
            let local_time = utc_time.with_timezone(&Local);
            local_time.format("%Y-%m-%d %H:%M:%S").to_string()
        })
    }

    /// 从 metadata 字段获取短哈希标识
    fn get_short_hash(&self) -> String {
        // 如果 metadata 字段存在且包含哈希值，则使用它
        if let Some(metadata) = &self.metadata {
            // 假设 metadata 字段直接存储哈希值
            // 如果 metadata 是 JSON 格式，可能需要解析
            metadata.clone()
        } else {
            // 如果没有 metadata，生成一个简单的备用标识
            format!("{:x}", self.id)
        }
    }

}

// 为 Log 实现 Display trait，用于简洁模式
impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let local_time = self.get_local_time().unwrap_or_else(|| self.timestamp.clone());
        /*
        let content_preview = if self.content.len() > 30 {
            format!("{}...", &self.content[..27])
        } else {
            self.content.clone()
        };
        */
        let content_preview = self.content.clone();

        write!(f, "{} | {}", local_time, content_preview)
    }
}

// ====================================================================
// 主处理函数 (The Conductor)
// 它的职责是：调用选择器，然后根据情况调用行动器或格式化器。
// ====================================================================
pub fn handle_get(args: &GetArgs, db_path: &PathBuf) -> rusqlite::Result<()> {
    let conn = Connection::open(db_path)?;

    // --- 第一步：筛选 ID ---
    let ids = select_log_ids(&conn, args);
    let ids = match ids {
        Ok(ids) => ids,
        Err(e) => panic!("{:?}", e),
    };

    // println!("IDs Found: {:?}", ids);

    if ids.is_empty() {
        println!("未找到匹配的日志。");
        return Ok(());
    }

    // --- 第二步：根据 ID 执行动作 ---

    // 检查是否有动作参数被提供
    let action_was_taken = if let Some(tag_to_add) = &args.add_tag {
        // ... (在这里实现 UPDATE tags 的逻辑) ...
        println!(
            "为 {} 条日志添加了标签 '{}' (功能待实现)",
            ids.len(),
            tag_to_add
        );
        true
    } else if let Some(_new_path) = &args.fix_path {
        // ... (在这里实现 UPDATE directory 的逻辑) ...
        println!(
            "将 {} 条日志的目录修改为 '{}' (功能待实现)",
            ids.len(),
            _new_path
        );
        true
    } else if args.delete {
        // ... (在这里实现 INSERT INTO backup 和 DELETE 的逻辑) ...
        println!("删除了 {} 条日志 (功能待实现)", ids.len());
        true
    } else {
        // 没有提供任何动作参数
        false
    };

    // --- 第三步：如果没有执行动作，则获取完整日志并打印 ---
    if !action_was_taken {
        let logs = get_logs_by_ids(&conn, &ids)?;

        // 根据参数选择输出格式（这里需要为 GetArgs 添加 format 字段）
        match args.format.as_str() {
            "tags" => format_detailed1(&logs),
            "iden" => format_detailed2(&logs),
            "json" => format_json(&logs),
            _ => format_compact(&logs), // 默认为简洁模式
        }
    }

    Ok(())
}


// ====================================================================
// [DONE] 输出结果可复用（原子测试其一）
// 阶段一：选择器 (Selector)
// 这个函数的唯一职责是：根据所有筛选参数，返回一个目标日志的 ID 列表。
// ====================================================================
fn select_log_ids(conn: &Connection, args: &GetArgs) -> AnyhowResult<Vec<i32>> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    // 构建 WHERE 子句的逻辑和之前完全一样
    if !args.all {
        let current_dir = std::env::current_dir()?
            .to_str()
            .context("无法将当前目录转换为字符串")?
            .to_string();

        if args.recursive {
            if args.global {
                conditions.push("(directory LIKE ? OR directory == 'global')".to_string());
            } else {
                conditions.push("directory LIKE ?".to_string());
            }
            params.push(Box::new(format!("{}%", current_dir)));
        } else {
            if args.global {
                conditions.push("(directory = ? OR directory == 'global')".to_string());
            } else {
                conditions.push("directory = ?".to_string());
            }
            params.push(Box::new(current_dir));
        }
    }

    if args.today {
        conditions.push("DATE(timestamp) = DATE('now', 'localtime')".to_string());
    } else if let Some(date) = &args.date {
        conditions.push("DATE(timestamp) = ?".to_string());
        params.push(Box::new(date.clone()));
    } else if let Some(hour) = &args.hour {
        conditions.push("timestamp >= datetime('now', ?)".to_string());
        params.push(Box::new(format!("-{} hours", hour)));
    } else if let Some(minute) = &args.minute {
        conditions.push("timestamp >= datetime('now', ?)".to_string());
        params.push(Box::new(format!("-{} minutes", minute)));
    }


    if let Some(tag) = &args.tag {
        conditions.push("tags LIKE ?".to_string());
        params.push(Box::new(format!("%{}%", tag)));
    }

    if let Some(keyword) = &args.keyword {
        conditions.push("content LIKE ?".to_string());
        params.push(Box::new(format!("%{}%", keyword)));
    }

    // 组装最终的 SELECT id 查询
    let where_clause = if !conditions.is_empty() {
        format!(" WHERE {}", conditions.join(" AND "))
    } else {
        "".to_string()
    };



    let mut sql = format!("SELECT id FROM logs{}", where_clause);
    if !args.reverse {
        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");
    } else {
        sql.push_str(" ORDER BY timestamp ASC LIMIT ?");
    }
    params.push(Box::new(args.num));

    // println!("Final SQL Code: [{:?}]", sql);

    // 执行查询并收集 ID
    let mut stmt = conn.prepare(&sql)?;
    let ids_iter = stmt.query_map(params_from_iter(params), |row| row.get(0))?;


    let ids = ids_iter.collect::<Result<Vec<i32>, _>>()?;
    Ok(ids)
}

// ====================================================================
// 超集采集器
// ====================================================================
fn get_logs_by_ids(conn: &Connection, ids: &[i32]) -> Result<Vec<Log>> {
    // 处理空 ID 列表的情况
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    // 创建占位符字符串
    let placeholders: Vec<String> = (0..ids.len()).map(|_| "?".to_string()).collect();
    let placeholders_str = placeholders.join(",");

    // 构建 SQL 查询
    let sql = format!(
        "SELECT id, timestamp, directory, content, tags, metadata, level
         FROM logs
         WHERE id IN ({})",
        placeholders_str
    );

    // 准备查询语句
    let mut stmt = conn.prepare(&sql)?;

    // 将 ID 转换为参数
    let params: Vec<&dyn rusqlite::ToSql> =
        ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

    // 执行查询并收集结果
    let log_iter = stmt.query_map(params.as_slice(), |row| Log::from_row(row))?;

    let mut logs = Vec::new();
    for log_result in log_iter {
        logs.push(log_result?);
    }

    Ok(logs)
}



// ====================================================================
// 阶段三：格式化输出 (Formatter)
// 这个函数的职责是：接收日志数据和显示选项，然后漂亮地打印它们。
// ====================================================================

/// 简洁模式：时间 | content 字段的前 30 个字
fn format_compact(logs: &[Log]) {
    for log in logs {
        println!("{}", log); // 使用 Display trait 的实现
    }
}

/// 详细模式1：时间 | 目录 | tags | 完整 content
fn format_detailed1(logs: &[Log]) {
    for log in logs {
        let local_time = log.get_local_time().unwrap_or_else(|| log.timestamp.clone());

        println!("Time : {}", local_time);
        println!("Dir  : {}", log.directory);

        if let Some(tags) = &log.tags {
            println!("Tags : {}", tags);
        }

        println!("Log  : {}", log.content);
        println!("---"); // 分隔线
    }
}

/// 详细模式2：时间 | 短哈希标识 | 完整 content
fn format_detailed2(logs: &[Log]) {
    for log in logs {
        let local_time = log.get_local_time().unwrap_or_else(|| log.timestamp.clone());
        let short_hash = log.get_short_hash();

        println!("Time : {}", local_time);
        println!("Hash : {}", short_hash);
        println!("Log  : {}", log.content);
        println!("---"); // 分隔线
    }
}

/// JSON 模式：打印 Json 样式的信息
fn format_json(logs: &[Log]) {
    match serde_json::to_string_pretty(&logs) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("JSON 序列化失败: {}", e),
    }
}

fn manual_time_parse(time_str: &str) -> Result<DateTime<FixedOffset>, ParseError> {
    // 移除纳秒部分后的精度，保留最多6位小数
    let simplified = if let Some(dot_pos) = time_str.find('.') {
        if let Some(z_pos) = time_str[dot_pos..].find(|c| c == 'Z' || c == '+') {
            let decimal_part = &time_str[dot_pos+1..dot_pos+z_pos];
            let limited_decimal = if decimal_part.len() > 6 {
                &decimal_part[..6] // 限制为6位小数
            } else {
                decimal_part
            };

            format!("{}.{}{}", 
                    &time_str[..dot_pos], 
                    limited_decimal,
                    &time_str[dot_pos+z_pos..])
        } else {
            time_str.to_string()
        }
    } else {
        time_str.to_string()
    };

    DateTime::parse_from_rfc3339(&simplified)
}

