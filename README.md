
# dlog

[原地址](https://github.com/ABdeeglr/dlog/blob/main/README.md)



A lightweight, developer-friendly CLI tool to log your daily progress directly in the terminal.

`dlog` is a Rust-based command-line utility that helps you track your work with contextual, searchable, and taggable logs tied to your project directories.

---

## ✨ Features

- **Quick Logging**: Log a one-liner with `-m`.
- **Interactive Logging**: Write multi-line entries with `dlog log`.
- **Directory-Aware**: Logs are automatically associated with your current working directory.
- **Tag Support**: Add comma-separated tags (e.g., `bug,ui,urgent`) to logs.
- **Powerful Filtering**:
  - Filter by tag (`-t work`)
  - Search by keyword (`-s "auth"`)
  - Filter by date (`--date 2025-10-13`)
  - Recursive mode (`-r`) to include subdirectories
- **Edit & Delete**: Modify or remove logs by ID using your `$EDITOR`.
- **SQLite Backend**: All logs stored in `~/.config/dlog/dlog.db`.

---

## 📦 Installation

```bash
git clone https://github.com/ABdeeglr/dlog.git
cd dlog
cargo install --path .
```

> Make sure `~/.cargo/bin` is in your `PATH`.

---

## 🚀 Usage

### 1. Initialize the database (first time only)
```bash
dlog init
```
Creates `~/.config/dlog/dlog.db`.

---

### 2. Record a log

**Quick log with message and tags:**
```bash
dlog log -m "Fixed login bug" -t "bug,auth"
```

**Interactive multi-line log:**
```bash
dlog log
# Type your log, then press Ctrl+D to save.
```

> Tags can be added interactively too by using `-t` even in interactive mode.

---

### 3. View logs

**Show latest 5 logs (default):**
```bash
dlog get
```

**Show last N logs:**
```bash
dlog get -n 10
```

**Include subdirectories:**
```bash
dlog get -r
```

**Filter by tag (exact or partial match):**
```bash
dlog get -t bug        # shows logs with tag "bug"
dlog get -t auth       # matches "auth" in "bug,auth"
```

**Search by keyword in content or tags:**
```bash
dlog get -s "login"
```

**Filter by date (YYYY-MM-DD):**
```bash
dlog get --date 2025-10-13
```

**Combine filters:**
```bash
dlog get -r -t work --date 2025-10-13 -n 20
```

> ✅ Tags are **always displayed** in output if present.

---

### 4. Edit or delete logs

**Edit a log by ID (opens `$EDITOR`):**
```bash
dlog fix 3
```

**Delete a log by ID (with confirmation prompt):**
```bash
dlog del 5
```

> Use `dlog get` first to find the log ID.

---

## 🛠️ Dependencies

- Rust & Cargo (for installation)
- An editor set via `$EDITOR` (e.g., `vim`, `nano`, `code`) for `fix`

---

## 📁 Data Storage

All logs are stored in:
```
~/.config/dlog/dlog.db
```
This is a standard SQLite database — you can inspect it with any SQLite client.

---
