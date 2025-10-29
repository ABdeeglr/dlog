# Get 命令的处理流程

## Get 参数的组成、设计原则及使用方式

Get 子命令的参数有很多，但是被明确分成了三个部分:
1. 动作参数
2. 查询参数
3. 输出选项参数

动作参数目前有 4 个：
1. `add_tag`, 用于追加选中的 log 的标签内容，对应 UPDATE 查询语句;
2. `fix_path`, 用于修改选中的 log 的路径内容，对应 UPDATE 查询语句;
3. `delete`, 用于批量删除（移动到备份表中，备份表会通过触发器自动清理多余的 log 内容）log，对应 DELETE 查询语句;
4. `force`, 与 `delete` 必须同时使用，否则不会执行删除操作;
5. 默认查询选项：如果上述参数均未提供，则表现为 SELECT 查询语句;

用户可以先不使用任何动作参数，来查看自己想要修改的 log, 然后添加动作参数进行操作.

这些参数的定义如下：
```rust
pub add_tag: Option<String>,
pub fix_path: Option<String>,
pub delete: bool,
pub force: bool,
```

**动作参数**决定了针对**查询参数**命中的 log 执行怎样的行为，而**查询参数**则决定命中哪些 log.

查询参数目前有 7 个，它们的定义如下: 

```rust
pub recursive: bool,
pub all: bool,
pub today: bool,
pub date: Option<String>,
pub hour: Option<u32>,
pub tag: Option<String>,
pub keyword: Option<String>,
```

这些查询参数分别代表：
1. 无任何参数: 查询与当前所在目录匹配的所有 log;
2. recursive: 查询当前所在的目录及其子目录下的所有 log;
3. today: 查询今日编写的 log;
4. hour: 查询 N 小时内编写的 log;
5. tag: 查询 log 标签部分包含特定关键词的 log;
6. keyword: 查询 log 内容部分包含特定关键词的 log;


最终，被查选到的 log 需要被以简洁地形式进行打印，在默认情况下，只打印记录 log 的时间和记录 log 的内容;

更进一步，你可以通过以下**格式化输出参数**对打印的内容进行修饰：
1. num: 对所有命中的 log, 按时间排序，打印前 N 条;
2. tags: 对所有命中的 log, 额外打印其标签部分;
3. identifer: 对所有命中的 log, 额外打印其短哈希标识, 在 Fix, Pop 子命令中（它们用于精确修饰特定的 log） 使用短哈希标识来确定需要更改的 log.

这些参数被定义为:

```rust
pub identifier: bool,
pub tags: bool,
pub num: u32,
```
用户可以通过 `dlog get --help` 了解这些参数的说明或是否可缩写.

总的来看，动作参数表现为互斥，你不能同时进行 DELETE 和 UPDATE, 而但动作参数与其他任意参数都是正交的，用户可以进行随意组合，从而精确地查询到自己想要的内容并加以操作.

## Get 子命令的处理流程

首先，`main.rs` 通过 `clap` 进行参数解析，从而得到一个 `Get {...}` 结构体，这个结构体将会被逐步消费.

**1 任务分发**

`handle_get` 函数将会首先在主函数解析到 get 子命令时被调用, 它的定义是 `pub fn handle_get(args: &GetArgs, db_path: &PathBuf) -> rusqlite::Result<()>`

**2 根据查询参数命中 logs**

`handle_get` 的第一步是根据查询参数来命中一组 log, 实现这一功能的是 `select_log_ids` 函数，它的定义是 `fn select_log_ids(conn: &Connection, args: &GetArgs) -> AnyhowResult<Vec<i32>>`.

`select_log_ids` 将会返回一个 `Vec<i32>` 类型的数组，由于在运行时可以保证 ids 的一致性（本软件并不是为多用户高并发的环境而设计的），所以 ids 足够有效。

**3 通过 ids 命中 logs 并采取措施**

`get_logs_by_ids` 函数能够通过上一步获得 `ids` 来查询完整的被命中的 logs, 它的定义是 `fn get_logs_by_ids(conn: &Connection, ids: &[i32]) -> Result<Vec<Log>>`.

这些被命中的 logs 被存储在 Vector 中，其中的每一个元素都被存储在预先定义好的 log 结构体中，方便后续操作，一般而言，就是根据格式化参数进行逐个打印。

**4 动作参数**

查询是默认的情况，如果用户没有进行设置动作参数才会进行，而在有动作参数的情况下，则会根据 ids 来执行特定的行为。在代码层面，动作参数是被预先处理的，只有当动作参数被跳过时，才会执行默认的查询，而在用户侧来看，总是先进行查询再采取动作。


