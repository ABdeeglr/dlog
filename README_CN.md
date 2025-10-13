# dlog


[原地址](https://github.com/ABdeeglr/dlog/blob/main/README.md)


一个轻量、开发者友好的命令行工具，让你在终端中轻松记录每日工作进展。

`dlog` 是一个基于 Rust 的命令行工具，帮助你通过**带上下文、可搜索、可打标签**的日志来追踪工作，日志自动与项目目录绑定。

---

## ✨ 功能特性

- **快速记录**：使用 `-m` 记录单行日志。
- **交互式记录**：运行 `dlog log` 编写多行详细内容。
- **目录感知**：日志自动关联到当前工作目录。
- **标签支持**：为日志添加逗号分隔的标签（如 `bug,ui,紧急`）。
- **强大筛选能力**：
  - 按标签过滤（`-t 工作`）
  - 按关键词搜索（`-s "登录"`）
  - 按日期筛选（`--date 2025-10-13`）
  - 递归模式（`-r`）包含子目录日志
- **编辑与删除**：通过 ID 使用 `$EDITOR` 修改或删除日志。
- **SQLite 存储**：所有日志保存在 `~/.config/dlog/dlog.db`。

---

## 📦 安装

```bash
git clone https://github.com/ABdeeglr/dlog.git
cd dlog
cargo install --path .
```

> 请确保 `~/.cargo/bin` 已加入系统 `PATH`。

---

## 🚀 使用方法

### 1. 初始化数据库（首次使用）
```bash
dlog init
```
将在 `~/.config/dlog/dlog.db` 创建数据库。

---

### 2. 记录日志

**快速记录（带消息和标签）：**
```bash
dlog log -m "修复了登录 Bug" -t "bug,认证"
```

**交互式多行记录：**
```bash
dlog log
# 输入日志内容，按 Ctrl+D 保存。
```

> 即使在交互模式下，也可通过 `-t` 添加标签。

---

### 3. 查看日志

**显示最新 5 条日志（默认）：**
```bash
dlog get
```

**显示最近 N 条：**
```bash
dlog get -n 10
```

**包含子目录日志：**
```bash
dlog get -r
```

**按标签过滤（支持精确或部分匹配）：**
```bash
dlog get -t bug        # 显示含 "bug" 标签的日志
dlog get -t 认证       # 匹配 "bug,认证" 中的 "认证"
```

**按关键词搜索（内容或标签中）：**
```bash
dlog get -s "登录"
```

**按日期筛选（格式：YYYY-MM-DD）：**
```bash
dlog get --date 2025-10-13
```

**组合多种筛选条件：**
```bash
dlog get -r -t 工作 --date 2025-10-13 -n 20
```

> ✅ 所有包含标签的日志在输出时**自动显示标签**。

---

### 4. 编辑或删除日志

**通过 ID 编辑日志（调用 `$EDITOR`）：**
```bash
dlog fix 3
```

**通过 ID 删除日志（需确认）：**
```bash
dlog del 5
```

> 可先用 `dlog get` 查看日志 ID。

---

## 🛠️ 依赖项

- Rust 与 Cargo（用于安装）
- 系统环境变量 `$EDITOR` 指定的编辑器（如 `vim`、`nano`、`code`），用于 `fix` 命令

---

## 📁 数据存储位置

所有日志保存在：
```
~/.config/dlog/dlog.db
```
这是一个标准的 SQLite 数据库，可用任意 SQLite 客户端查看。

---
