// src/commands/get.rs

use crate::GetArgs;
use anyhow::{Context, Result as AnyhowResult};
use rusqlite::{params_from_iter, Connection, Result, Row, ToSql};
use std::path::PathBuf;
use chrono::{DateTime, FixedOffset, Offset, ParseError, Local};

// 定义一个 LogEntry 结构体，用于在最后阶段存放和显示日志数据
#[derive(Debug, Clone)]
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

        match get_logs_by_ids(&conn, &ids) {
            Ok(logs) => {
                for log in logs {
                    println!("Found log: {:#?}", log);
                }
            }
            Err(e) => eprintln!("Error querying logs: {}", e),
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
            conditions.push("directory LIKE ?".to_string());
            params.push(Box::new(format!("{}%", current_dir)));
        } else if args.global {
            conditions.push("OR directory == 'global'".to_string());
        } else {
            conditions.push("directory = ?".to_string());
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
    sql.push_str(" ORDER BY timestamp DESC LIMIT ?");
    params.push(Box::new(args.num));

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
/*
fn display_local_time(utc_time_str: &str) {
    // 尝试多种解析方式
    let parse_result = DateTime::parse_from_rfc3339(utc_time_str)
        .or_else(|_| {
            // 如果标准解析失败，尝试更宽松的解析
            DateTime::parse_from_str(utc_time_str, "%+")
        })
        .or_else(|_| {
            // 如果还失败，尝试手动处理时区信息
            manual_time_parse(utc_time_str)
        });
    
    match parse_result {
        Ok(utc_time) => {
            // 转换为本地时区
            let local_time = utc_time.with_timezone(&Local);
            
            // 友好显示格式
            println!("[用户显示] 本地时间: {}", local_time.format("%Y年%m月%d日 %H时%M分%S秒"));
            println!("[用户显示] 详细格式: {}", local_time.format("%A, %B %d, %Y at %I:%M:%S %p"));
            println!("[用户显示] 简洁格式: {}", local_time.format("%Y-%m-%d %H:%M:%S"));
            
            // 显示时区信息
            let timezone_offset = local_time.offset().fix().local_minus_utc() / 3600;
            println!("[用户显示] 时区: UTC{}{:02}", 
                     if timezone_offset >= 0 { "+" } else { "-" }, 
                     timezone_offset.abs());
        }
        Err(e) => {
            eprintln!("错误: 无法解析时间字符串 '{}': {}", utc_time_str, e);
        }
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
*/
