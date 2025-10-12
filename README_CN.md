# README.md

## dlog

![Demo](https://github.com/ABdeeglr/dlog/blob/main/demo.gif)

### 再也不用到处写 README 文件来记录你的操作

A command-line tool for developers to easily log their work progress.

`dlog` 是一个简单而强大的命令行工具，使用 Rust 编写，旨在帮助开发者直接在终端中记录日常任务和项目进展，并实现方便的搜索。

### 功能特性

- **快速记录：** 只需一个简单的命令即可快速记录。
- **交互模式：** 支持多行输入，方便记录更详细的内容。
- **项目感知：** 日志与当前工作目录关联，让你能轻松检索特定项目下的记录。
- **搜索与筛选：** 支持按目录、标签和数量来检索日志。


### 安装

由于 `dlog` 是一个 Rust 项目，你可以使用 `cargo` 进行安装：

1. 克隆项目到本地：
   `git clone https://github.com/ABdeeglr/dlog.git`
2. 进入项目目录：
   `cd dlog`
3. 编译并安装：
   `cargo install --path .`

`dlog` 可执行文件会被安装到你的 Cargo bin 目录下（`~/.cargo/bin/`），该目录通常已在你的系统 PATH 中。

### 使用

#### 1. 初始化数据库

首先，你需要创建用于存储所有日志的数据库文件。
`dlog init`

数据库文件将被创建在 `~/.config/dlog/dlog.db`。


#### 2. 记录日志

你有两种方式来记录日志：

- **快速记录：** 记录简短的单行消息。
  `dlog log -m "实现了用户认证功能。"`

- **交互记录：** 用于记录多行、详细的日志。
  `dlog log`
  _你会被引导进入交互式输入界面。输入完成后，按 `Ctrl + D` 结束并保存。_


#### 3. 查看日志

从命令行中检索你的日志。

- **查看最新一条日志：**
  `dlog get`

- **查看最新的 5 条日志：**
  `dlog get -n 5`

- **查看当前目录及其子目录下的所有日志：**
  `dlog get -r`

- **查看日志并显示标签：**
  `dlog get -t`

