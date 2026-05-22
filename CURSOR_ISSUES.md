# Cursor 支持问题分析与修复

## 问题总结

### 1. Codex 时间戳问题 ✅ 已修复

**问题**：Codex 的 69 个会话的 `first_time` 和 `last_time` 字段都是空的，导致 summary 命令无法统计。

**原因**：`load_codex_session_metadata` 函数没有像 `extract_codex_session_info` 那样的文件 mtime 回退机制。

**修复**：在 `src/provider/codex.rs` 中添加了文件 mtime 回退逻辑：

```rust
// Fallback: use file mtime if no timestamps found in JSONL
if first_time.is_empty() || last_time.is_empty() {
    if let Ok(metadata) = path.metadata() {
        if let Ok(mtime) = metadata.modified() {
            let dt: chrono::DateTime<chrono::Utc> = mtime.into();
            let ts = dt.to_rfc3339();
            if first_time.is_empty() {
                first_time = ts.clone();
            }
            if last_time.is_empty() {
                last_time = ts;
            }
        }
    }
}
```

**验证**：
```bash
$ cargo run -- summary rnproject --date 2026-05-21 --provider codex
# 现在可以正确统计 Codex 会话了
```

### 2. Cursor Workspace 映射问题 ⚠️ 已隔离，完整修复待实现

**问题**：rnproject 的 workspace (`f1fb5eb4b23dd2dedb8b09609cb09e6b`) 没有被识别。

**调查发现**：

1. **workspace.json 正确**：
   ```json
   {"folder": "file:///Users/zhangjiantao/Documents/juke/rnproject"}
   ```

2. **工作区数据库存在**：
   - `state.vscdb` (180KB)
   - `state.vscdb-wal` (4MB) - 说明有最近的活动
   - `state.vscdb-shm` (32KB)

3. **但 `allComposers` 为空**：
   ```bash
   $ sqlite3 state.vscdb "SELECT value FROM ItemTable WHERE key = 'composer.composerData'" | jq '.allComposers | length'
   0
   ```

4. **全局数据库中有所有 composers**：
   ```bash
   $ sqlite3 globalStorage/state.vscdb "SELECT COUNT(*) FROM cursorDiskKV WHERE key LIKE 'composerData:%'"
   # 返回大量 composer
   ```

**结论**：Cursor 的存储策略不一致：
- 有些 workspace 把 composer 列表存在工作区数据库的 `composer.composerData` 中
- 有些 workspace（如 rnproject）的工作区数据库中 `allComposers` 为空
- 所有 composer 的详细数据都存储在全局数据库的 `composerData:<id>` 键中
- **但全局数据库中的 composer 没有明确的 workspace 映射字段**

**当前处理**：不把 `allComposers` 为空的 workspace 作为正常项目列出。

```rust
let composers = match read_workspace_composers(&ws_db_path) {
    Ok(c) => c,
    Err(_) => continue,
};

if active.is_empty() {
    continue;
}
```

**原因**：如果只是保留空 workspace，会让 `list --provider cursor` 输出大量 `0 sessions` 项目，但 `sessions <project>` 仍然没有可加载会话。这会污染正常列表，并没有真正解决 Cursor 全局 composer 到 workspace 的映射问题。

**效果**：
```bash
$ cargo run -- list --provider cursor | grep rnproject
# 无输出，直到实现全局 composer 映射
```

现在 rnproject 不会以空项目形式误报。完整支持仍需实现下面的方案 A 或方案 C。

### 3. 完整修复方案（待实现）

要完全解决 Cursor 的问题，需要实现以下功能：

#### 方案 A：全局 Composer 扫描 + 路径推断

1. **扫描全局数据库中的所有 composers**：
   ```sql
   SELECT key, value FROM cursorDiskKV WHERE key LIKE 'composerData:%'
   ```

2. **对每个 composer，读取其 bubble 内容**：
   ```sql
   SELECT value FROM cursorDiskKV WHERE key LIKE 'bubbleId:<composer-id>:%'
   ```

3. **从 bubble 内容中提取文件路径**：
   - Tool calls 中的 `file_path` 参数
   - 代码块中的文件引用
   - 上下文中的文件路径

4. **将文件路径映射到 workspace**：
   - 读取所有 workspace.json
   - 匹配文件路径前缀
   - 将 composer 归属到对应的 workspace

#### 方案 B：按消息时间戳统计（推荐）

当前 `summary` 命令使用 `session.first_time` 来过滤日期：

```rust
fn session_date(session: &Session) -> Option<NaiveDate> {
    chrono::DateTime::parse_from_rfc3339(&session.first_time)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Local).date_naive())
}
```

**问题**：如果用户今天在旧会话中继续对话，`first_time` 是旧日期，不会被统计。

**建议**：
1. 添加 `--by-message-time` 标志
2. 按消息的时间戳过滤，而不是会话创建时间
3. 即使是旧会话，只要有今天的消息，就应该被统计

```rust
fn has_messages_in_range(messages: &[Message], range: &DateRange) -> bool {
    messages.iter().any(|m| {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&m.timestamp) {
            let date = dt.with_timezone(&chrono::Local).date_naive();
            date >= range.start && date <= range.end
        } else {
            false
        }
    })
}
```

#### 方案 C：读取更多的键

根据用户提供的信息，Cursor 可能在以下键中存储聊天数据：

**全局数据库**：
- `workbench.panel.aichat.view.aichat.chatdata`
- `workbench.panel.composerChatViewPane.<uuid>.*`
- `conversationClassificationScoredConversations`

**工作区数据库**：
- `composer.composerData`（已读取）
- `aiService.generations`
- `aiService.prompts`
- `workbench.backgroundComposer.workspacePersistentData`（已检查，但只有 git 状态）

需要检查这些键是否包含额外的聊天数据。

### 4. WAL 文件处理

当前代码使用 `rusqlite` 的 `SQLITE_OPEN_READ_ONLY` 模式打开数据库，这应该会自动读取 WAL 文件。

但为了确保读取最新数据，可以在打开连接后执行：
```rust
conn.execute("PRAGMA wal_checkpoint(PASSIVE)", [])?;
```

或者在查询前：
```rust
conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
```

## 测试验证

### Codex 修复验证

```bash
# 1. 检查 Codex 会话是否有时间戳
cargo run -- sessions rnproject --provider codex --json | jq '.[] | {id, first_time, last_time}' | head -20

# 2. 测试 summary 命令
cargo run -- summary rnproject --date 2026-05-21 --provider codex

# 3. 测试日期范围
cargo run -- summary rnproject --range 2026-05-20..2026-05-22 --provider codex
```

### Cursor 修复验证

```bash
# 1. 检查 rnproject 是否出现在列表中
cargo run -- list --provider cursor | grep rnproject

# 2. 检查会话数量（当前为 0）
cargo run -- sessions rnproject --provider cursor

# 3. 手动检查全局数据库中的 composer
sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
  "SELECT COUNT(*) FROM cursorDiskKV WHERE key LIKE 'composerData:%'"
```

## 下一步行动

1. **立即可用**：Codex 修复已完成，可以正常使用
2. **Cursor 正常列表保持干净**：空 composer workspace 不会被误报为可用项目
3. **完整修复**：需要实现方案 A 或 C；方案 B 可作为 summary 的独立增强

## 相关文件

- `src/provider/codex.rs` - Codex 时间戳修复
- `src/provider/cursor.rs` - Cursor workspace 映射修复
- `src/summary.rs` - Summary 命令逻辑（需要修改以支持按消息时间戳过滤）
