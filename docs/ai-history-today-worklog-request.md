# ai-history 今日工作聚合命令修改需求

## 背景

`ai-history` 本体是 Rust CLI。现在如果想总结某个项目今天的工作，需要先分别查询多个 provider，再用外部脚本整理 JSON：

```bash
ai-history sessions "<project>" --json
ai-history sessions "<project>" --provider cursor --json
ai-history sessions "<project>" --provider codex --json
```

然后还要合并、按日期筛选、排序、提取标题。对于很大的 session，如果再调用 `show <session-id>`，还容易超时。

希望把这类常用整理能力直接集成进 `ai-history`，让 CLI 直接输出今日工作标题或摘要。

---

## 总目标

新增一个纯规则处理的今日工作聚合能力，不依赖 LLM，不调用外部 AI。

核心命令：

```bash
ai-history today [project]
```

示例：

```bash
ai-history today . --titles
ai-history today . --summary
ai-history today . --json
ai-history today . --date 2026-06-03 --titles
ai-history today . --provider codex --titles
```

---

## 目标 1：新增 `today` 子命令

新增命令：

```bash
ai-history today [project]
```

用途：输出指定项目今天的工作记录。

`[project]` 支持：

```bash
ai-history today .
ai-history today "/Users/zhangjiantao/Documents/juke/rnproject"
ai-history today rnproject
```

如果不传 project，则默认当前工作目录：

```bash
ai-history today
```

等价于：

```bash
ai-history today .
```

---

## 目标 2：默认跨 provider 聚合

`today` 默认查询所有 provider：

- `claude`
- `cursor`
- `codex`

也就是说：

```bash
ai-history today .
```

默认等价于查询全部 provider，而不是只查 Claude。

可额外支持：

```bash
ai-history today . --provider claude
ai-history today . --provider cursor
ai-history today . --provider codex
ai-history today . --all-providers
```

规则：

- 默认就是 `--all-providers`
- 如果传了 `--provider`，只查指定 provider

---

## 目标 3：支持标题模式

新增参数：

```bash
ai-history today . --titles
```

只输出今天工作的标题列表，不需要详细内容。

期望输出示例：

```text
TODAY WORK TITLES — ~/Documents/juke/rnproject
════════════════════════════════════════════
- App Store 审核被拒原因排查与重新提交流程确认
- 官网 Support 页面创建与 /support/ 访问验证
- Android 多渠道 APK 打包与渠道名读取实验
- iOS 微信分享 SDK 注册失败排查
- iOS 微信分享回调 errCode 日志修复
- Figma 设计稿访问权限排查
- 社区详情页 can-view 权限与锁定蒙层问题
- GroupDetailPage 二级评论缩进异常修复
════════════════════════════════════════════
```

标题来源优先级建议：

1. session 自带的 `summary`
2. 第一条非系统用户消息
3. 关键文件、API、页面名推断
4. 如果无法判断，使用 fallback：

```text
未命名会话：<provider>/<session_id前8位>
```

不要把这些内容当作标题：

- `# AGENTS.md instructions...`
- `<INSTRUCTIONS>`
- CodeGraph 规则
- environment context
- untrusted evidence 审批文本
- `<command-message>`
- `<system-reminder>`

---

## 目标 4：支持摘要模式

新增参数：

```bash
ai-history today . --summary
```

输出比标题更详细的今日工作总结。

示例：

```text
TODAY WORK SUMMARY — ~/Documents/juke/rnproject
════════════════════════════════════════════

1. iOS 微信分享 SDK 注册失败排查
   Provider: codex
   Session: 019e8b96
   Time: 11:45 - 18:33
   Files:
   - ios/afgroup/AppDelegate.swift
   Summary:
   - 排查微信 SDK 注册失败原因
   - 修复 iOS 微信回调未转发到 react-native-wechat-lib 的问题
   - 让 JS 侧可以打印 SendMessageToWX.Resp errCode

2. Android 多渠道 APK 打包与渠道名读取实验
   Provider: codex
   Session: 019e8852
   Summary:
   - 设计每个渠道一个 APK 的实验方案
   - 在包内写死 channel
   - 编译生成不同渠道 APK
```

---

## 目标 5：支持 JSON 输出

新增：

```bash
ai-history today . --json
ai-history today . --titles --json
ai-history today . --summary --json
```

推荐 JSON 结构：

```json
[
  {
    "title": "iOS 微信分享 SDK 注册失败排查",
    "provider": "codex",
    "session_id": "019e8b96",
    "session_id_full": "019e8b96-04e1-7843-b424-8128783365a6",
    "project": "/Users/zhangjiantao/Documents/juke/rnproject",
    "first_time": "2026-06-03T03:48:08.720Z",
    "last_time": "2026-06-03T06:29:04.238Z",
    "message_count": 56,
    "files_touched": [
      "ios/afgroup/AppDelegate.swift"
    ],
    "summary": [
      "排查微信 SDK 注册失败原因",
      "修复 iOS 微信回调未转发到 react-native-wechat-lib 的问题"
    ]
  }
]
```

如果 `--titles --json`，也可以简化为：

```json
[
  {
    "title": "iOS 微信分享 SDK 注册失败排查",
    "provider": "codex",
    "session_id": "019e8b96"
  }
]
```

---

## 目标 6：支持日期参数

默认是今天，同时支持指定日期：

```bash
ai-history today . --date 2026-06-03
```

后续可以扩展更通用的 `worklog`：

```bash
ai-history worklog . --since today
ai-history worklog . --since yesterday
ai-history worklog . --since 2026-06-01
ai-history worklog . --from 2026-06-01 --to 2026-06-03
```

本次如果实现成本较高，可以先只实现：

```bash
ai-history today . --date YYYY-MM-DD
```

---

## 目标 7：本地时区日期过滤规则

判断某个 session 是否属于指定日期：只要满足以下任一条件，就纳入结果：

- `first_time` 在指定日期
- `last_time` 在指定日期
- session 时间区间跨过指定日期

例如：

```text
first_time = 2026-06-02 23:00
last_time  = 2026-06-03 01:00
```

应该算作 `2026-06-03` 的工作。

日期判断应使用本地时区，不要只按 UTC 日期判断。

---

## 目标 8：标题清洗和去重

### 去重规则

做基础去重：

- 同 provider + 同 session id 去重
- 完全相同标题去重
- 去掉空标题
- 去掉明显的系统提示标题

### 标题清洗规则

- 去掉 markdown 图片标记
- 去掉 `[Image #1]`
- 去掉过长代码块
- 去掉换行
- 连续空白压缩为一个空格
- 标题最长建议 60 个中文字符左右，超过就截断或生成更短标题

---

## 目标 9：不要强依赖 LLM

这个功能应该是纯 CLI / 规则处理，不要调用外部 LLM。

标题生成可以基于：

- session summary
- user message
- touched files
- command args
- provider metadata

不要调用 OpenAI、Claude 或本地模型。

如果将来要支持 AI 总结，可以单独做：

```bash
ai-history today . --ai-summary
```

本次不要实现 `--ai-summary`。

---

## 建议实现方式

请先查看当前项目结构，重点找：

- CLI 参数定义位置
- `sessions` 子命令实现
- provider 枚举
- Claude / Cursor / Codex session 解析逻辑
- summary 生成逻辑
- JSON 输出逻辑

然后复用现有 `sessions` 的数据结构，新增统一的 `WorklogEntry` 或类似结构。

建议结构：

```rust
struct WorklogEntry {
    title: String,
    provider: String,
    session_id: String,
    session_id_full: String,
    project: String,
    first_time: Option<DateTime<Local>>,
    last_time: Option<DateTime<Local>>,
    message_count: usize,
    files_touched: Vec<String>,
    summary: Vec<String>,
}
```

核心流程：

```text
parse args
resolve project
load sessions from selected providers
filter by date in local timezone
clean sessions
extract title
extract summary bullets
dedupe
sort by first_time
render text or json
```

排序：

- 默认按 `first_time` 升序
- 如果没有 `first_time`，按 `last_time`
- 都没有则排最后

---

## 命令示例与预期行为

### 当前项目今日标题

```bash
ai-history today . --titles
```

输出标题列表。

### 当前项目今日摘要

```bash
ai-history today . --summary
```

输出每个工作项的详情。

### JSON 输出

```bash
ai-history today . --json
```

输出结构化 JSON。

### 指定日期

```bash
ai-history today . --date 2026-06-03 --titles
```

输出 2026-06-03 当天标题。

### 指定 provider

```bash
ai-history today . --provider codex --titles
```

只输出 Codex 的今日工作标题。

---

## 测试要求

请增加或更新测试，覆盖：

1. `today --titles` 可以输出今天标题
2. 默认查询所有 provider
3. `--provider codex` 只输出 Codex
4. 跨 UTC 日期但本地属于今天的 session 能被包含
5. 系统提示类 summary 不会作为标题
6. 重复标题会去重
7. `--json` 输出合法 JSON
8. project 参数为空时默认当前目录

如果项目已有 snapshot test，可以加 snapshot。

---

## 验证命令

修改完成后请运行：

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
```

如果 `clippy` 当前项目本身已有历史 warning，可以如实说明。

本地手动验证：

```bash
cargo run -- today . --titles
cargo run -- today . --summary
cargo run -- today . --json
cargo run -- today . --date 2026-06-03 --titles
cargo run -- today . --provider codex --titles
```

---

## 兼容性要求

不要破坏已有命令：

```bash
ai-history list
ai-history sessions
ai-history show
ai-history search
ai-history export
```

已有 `sessions --json` 的输出格式不要改，除非必须；如果要改，需要保持向后兼容。

---

## 最终交付说明

完成后请输出：

1. 修改了哪些文件
2. 新增了哪些命令参数
3. 示例输出
4. 测试结果
5. 是否需要重新安装，例如：

```bash
cargo install --path .
```
