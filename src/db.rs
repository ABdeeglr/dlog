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

use rusqlite::ffi::ErrorCode;
use rusqlite::{Connection, Result};
use std::path::Path;

pub fn _initialize_db(db_path: &Path) -> Result<()> {
    // 检查父目录是否存在，如果不存在则创建
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
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

    Ok(())
}
