# 未来蓝图 (v0.2.0): 标签系统重构备忘录

## 核心目标 1

将标签从当前存储在 logs.tags 字段的“扁平化、逗号分隔的字符串”模型，升级为“结构化、关系化”的模型，以支持多标签、标签层级（树状）和高效查询。

### 1. 数据库结构变更 (Migration to v3):

创建 tags 表: 这张表是所有标签的“户口本”。

```SQL
CREATE TABLE tags (
    tag_id      INTEGER PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE, -- 标签名必须唯一
    parent_id   INTEGER,              -- 指向自己的 tag_id，用于实现树状结构
    FOREIGN KEY (parent_id) REFERENCES tags(tag_id)
);
```
parent_id 是实现“树状”的关键。例如，“rust” 标签的 parent_id 可以是 “programming” 标签的 tag_id。

创建 log_tags 关联表 (Junction Table): 这张表是连接 logs 和 tags 的“桥梁”，因为它俩是“多对多”关系（一条日志可以有多个标签，一个标签可以用于多条日志）。

```SQL
CREATE TABLE log_tags (
    log_id      INTEGER NOT NULL,
    tag_id      INTEGER NOT NULL,
    PRIMARY KEY (log_id, tag_id), -- 确保同一条日志不会重复关联同一个标签
    FOREIGN KEY (log_id) REFERENCES logs(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

ON DELETE CASCADE 意味着如果一条日志或一个标签被删除，所有与之相关的关联记录也会被自动清除。

修改 logs 表: logs 表不再需要 tags 字段，可以将其移除。

```SQL
-- In migration script
ALTER TABLE logs DROP COLUMN tags;
```

### 2. 程序逻辑变更:

log 命令: 当用户输入 -t "rust,dev" 时，程序需要：

遍历 ["rust", "dev"] 这个列表。

对每个标签名，去 tags 表里查询 tag_id。如果标签不存在，就先在 tags 表里创建一个新的。

获取到新日志的 log_id 和所有标签的 tag_id 后，在 log_tags 表里插入关联记录，例如 (new_log_id, rust_tag_id) 和 (new_log_id, dev_tag_id)。

get 命令: 当用户 --tag "rust" 时，查询会变得更高效和精确：

```SQL
SELECT l.* FROM logs l
JOIN log_tags lt ON l.id = lt.log_id
JOIN tags t ON lt.tag_id = t.tag_id
WHERE t.name = 'rust';
```

### 3. 关于触发器:

在这种模型下，通常不需要额外的触发器。FOREIGN KEY 约束和 ON DELETE CASCADE 已经处理了大部分数据一致性的问题。应用程式的逻辑负责正确地创建和关联数据。
