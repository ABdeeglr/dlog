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

// src/main.rs

// 声明模块
mod commands;
mod db;

// 引入依赖
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// 初始化 dlog 环境，在 ~/.config/dlog/ 目录下创建数据库文件。
#[derive(Args, Debug)]
pub struct InitArgs {
    /// [保留] 用于未来的数据库结构升级
    #[arg(short, long)]
    pub upgrade: bool,
}

/// 记录一条新的日志。
///
/// 如果不提供 -m/--message 参数，程序将进入交互式输入模式，
/// 允许多行输入，直到按下 Ctrl+D 结束。
#[derive(Args, Debug)]
pub struct LogArgs {
    /// 提供一条短消息直接记录 (类似 git commit -m)
    #[arg(short, long)]
    pub message: Option<String>,

    /// 为此条日志附加一个或多个标签 (逗号分隔)
    #[arg(short, long, value_name = "TAGS")]
    pub tags: Option<String>,

    /// 将此条日志记为全局日志，不与任何特定目录关联
    #[arg(short = 'g', long)]
    pub global: bool,
}

/// 查询并显示已记录的日志。
///
/// 提供了丰富的筛选条件，可以组合使用来精确定位你需要的日志。
/// 还可以附加--add-tag, --fix-path, --delete等动作参数来对查询结果进行批量操作。
#[derive(Args, Debug)]
pub struct GetArgs {
    /* 决定 SELECT/UPDATE/DELETE 阶段的SQL参数 */
    /// [动作] 为所有查询命中的日志追加一个新标签
    #[arg(long, group = "action", value_name = "TAG")]
    pub add_tag: Option<String>,

    /// [动作] 批量修改所有查询命中的日志的目录信息
    #[arg(long, group = "action", value_name = "PATH")]
    pub fix_path: Option<String>,

    /// [动作] 将查询命中的日志移动到备份区
    #[arg(long, requires = "force", group = "action")]
    pub delete: bool,

    /// [安全] 执行危险操作如 --delete 时一同使用，以确认删除操作
    #[arg(long)]
    pub force: bool,

    /* 决定 WHERE 子句的参数 其一 目录字段筛选 */
    /// 递归查询，匹配当前目录及其所有子目录下的日志。
    #[arg(short, long)]
    pub recursive: bool,

    /// 查询所有日志，忽略当前工作目录的限制。
    #[arg(long)]
    pub all: bool,

    /* 决定 WHERE 子句的参数 其二 时间字段筛选 */
    /// 筛选今天的日志
    #[arg(long)]
    pub today: bool,

    /// 按特定日期筛选 (格式: YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub date: Option<String>,

    /// 筛选最近 N 小时内的日志
    #[arg(short = 'H', long, value_name = "HOURS")]
    pub hour: Option<u32>,

    /* 决定 WHERE 子句的参数 其三 内容匹配筛选 */
    /// 筛选包含特定标签的日志
    #[arg(long, value_name = "TAG")]
    pub tag: Option<String>,

    /// 筛选内容中包含特定关键字的日志
    #[arg(long, value_name = "KEYWORD")]
    pub keyword: Option<String>,

    /* 最终格式化输出使用的参数 */
    /// 在结果中显示每条日志的唯一标识符 (短哈希)
    #[arg(short, long)]
    pub identifier: bool,

    /// 在查询结果中一并显示每条日志的标签。
    #[arg(short = 't', long, visible_alias = "show-tags")]
    pub tags: bool,

    /// 最终显示最新的 N 条日志 (默认为 1)
    #[arg(short, long, default_value_t = 1)]
    pub num: u32,
}

/// 精确修改某一条已存在的日志。
///
/// 通过唯一的标识符 (短哈希) 定位到具体某一条日志，然后对其内容、标签或目录进行更新。
#[derive(Args, Debug)]
pub struct FixArgs {
    /// [必需] 提供要修改的日志的唯一标识符 (短哈希)
    pub identifier: String,

    /// 更新/覆盖日志的标签
    #[arg(short, long)]
    pub tag: Option<String>,

    /// 更新/覆盖日志的内容
    #[arg(short, long)]
    pub content: Option<String>,

    /// 更新日志的目录信息
    #[arg(short, long)]
    pub directory: Option<String>,
}

/// 精确地将一条或多条日志移动到备份区。
///
/// 根据提供的唯一标识符 (短哈希) 精确查找，可以一次提供多个标识符进行批量操作。
#[derive(Args, Debug)]
pub struct PopArgs {
    /// [必需] 提供一个或多个要移除的日志的唯一标识符 (短哈希)
    #[arg(required = true, num_args = 1..)]
    pub identifiers: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    Init(InitArgs),
    Log(LogArgs),
    Get(GetArgs),
    Fix(FixArgs),
    Pop(PopArgs),
}

#[derive(Parser)]
#[command(author="ABdeeglr Ramsay", version="0.1.1", about="一个为开发者设计的、轻量级的命令行日志工具。", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/**
 * # 3 主函数
 *
 * 进行参数解析和任务分发
 */

fn main() -> Result<()> {
    let cli = Cli::parse();

    let db_path = {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let mut path = PathBuf::from(&home_dir);
        path.push(".config/dlog/dlog.db");
        path
    };

    // 如果用户在数据库尚未初始化的情况下执行子命令，则给出初始化提示
    if !db_path.exists() && !matches!(&cli.command, Commands::Init(_)) {
        eprintln!("Error: database instance not initialized. Run `dlog init` firstly.");
        return Ok(());
    }

    // 在运行时检查版本
    if db_path.exists() {
        db::run_migrations(&db_path)?;
    }

    // 核心的模式匹配与分发逻辑
    match &cli.command {
        Commands::Init(args) => commands::init::handle_init(args, &db_path)?,
        Commands::Log(args) => commands::log::handle_log(args, &db_path)?,
        Commands::Get(args) => commands::get::handle_get(args, &db_path)?,
        Commands::Fix(args) => commands::fix::handle_fix(args, &db_path)?,
        Commands::Pop(args) => commands::pop::handle_pop(args, &db_path)?,
    };

    Ok(())
}
