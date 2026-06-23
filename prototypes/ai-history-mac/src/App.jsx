import { useEffect, useMemo, useState } from "react";
import {
  Activity,
  Archive,
  Bot,
  Box,
  Brain,
  CalendarDays,
  Check,
  ChevronRight,
  Clipboard,
  Clock,
  Code2,
  Copy,
  Database,
  Download,
  FileCode2,
  FileText,
  Folder,
  Gauge,
  GitBranch,
  HardDrive,
  History,
  KeyRound,
  Layers3,
  LayoutDashboard,
  ListFilter,
  MessageSquareText,
  PanelLeftClose,
  Play,
  RefreshCcw,
  Search,
  Settings,
  ShieldCheck,
  Sparkles,
  TerminalSquare,
  Workflow,
  Zap,
} from "lucide-react";

const providers = [
  { id: "all", label: "All" },
  { id: "claude", label: "Claude" },
  { id: "codex", label: "Codex" },
  { id: "cursor", label: "Cursor" },
];

const i18n = {
  EN: {
    appTitle: "AI History",
    appSubtitle: "Local Memory Monitor",
    navDashboard: "Dashboard",
    navSearch: "Search",
    navSessions: "Sessions",
    navContext: "Context Builder",
    navWorklog: "Worklog",
    navWorkflows: "Workflows",
    navProviders: "Providers",
    navSettings: "Settings",
    collapse: "Collapse",
    localOnly: "Local only",
    upToDate: "Up to date",
    threeProviders: "3 providers",
    searchPlaceholder: "Search sessions, files, tools, decisions...",
    refresh: "Refresh",
    live: "Live",
    dashboardSubtitle: "Local overview of Claude Code, Codex CLI, and Cursor history.",
    totalSessions: "Total Sessions",
    activeToday: "3 active today",
    projects: "Projects",
    acrossProviders: "across 3 providers",
    messages: "Messages",
    indexedLocally: "indexed locally",
    todayWork: "Today Work",
    summariesReady: "4 summaries ready",
    digestCache: "Digest Cache",
    errorSessions: "Error Sessions",
    hitRate: "hit rate",
    privacy: "Privacy",
    noUpload: "no upload by default",
    recentSessions: "Recent Sessions",
    viewAll: "View all",
    open: "Open",
    filters: "Filters",
    query: "Query",
    searchMode: "Search mode",
    anyTerms: "Any terms",
    allTerms: "All terms",
    contextWindow: "Context window",
    contextWindowHelp: "2 messages before and after each hit",
    results: "Results",
    relevanceSort: "relevance sort",
    noResults: "No matching sessions",
    noResultsHelp: "No matching local history was returned by the ai-history CLI.",
    sampleData: "Recent local sessions",
    searchSubtitle: "Search local Claude Code, Codex CLI, and Cursor history.",
    prototypeNotice: "Real local data mode",
    prototypeNoticeHelp: "Dashboard, sessions, providers, worklog, workflows, and search read from the local ai-history CLI. No mock data is shown.",
    realSearch: "Real local search",
    searchLoading: "Searching local AI history...",
    searchError: "Local search failed",
    searchIdle: "Enter a query to search real local AI history. Recent local sessions are shown before searching.",
    loadingLocalHistory: "Loading local history...",
    localHistoryError: "Local history load failed",
    noLocalSessions: "No local sessions found",
    sessionsSubtitle: "Browse projects, inspect transcripts, and generate reusable memory.",
    export: "Export",
    transcript: "Transcript",
    digest: "Digest",
    files: "Files",
    tools: "Tools",
    intent: "Intent",
    keyDecisions: "Key Decisions",
    codeChanges: "Code Changes",
    remainingIssues: "Remaining Issues",
    copyDigest: "Copy Digest",
    enhance: "Enhance",
    contextSubtitle: "Turn historical sessions into clean context for the next AI coding run.",
    sources: "Sources",
    selected: "selected",
    contextPack: "Context Pack",
    output: "Output",
    estimatedTokens: "estimated tokens",
    copyContext: "Copy Context",
    exportMarkdown: "Export Markdown",
    worklogSubtitle: "Summarize local AI coding activity by project, date, and provider.",
    summary: "Summary",
    todayWorkTitles: "Today Work Titles",
    aiWorkSummary: "AI Work Summary",
    copy: "Copy",
    workflowsSubtitle: "Find repeated manual workflows worth packaging as skills or automation.",
    candidates: "Candidates",
    frequency: "Frequency",
    recommendation: "Recommendation",
    coverage: "Coverage",
    previewSkill: "Preview Skill Draft",
    providersSubtitle: "Inspect local data sources, scan state, and diagnostics.",
    connected: "connected",
    rescan: "Rescan",
    diagnostics: "Diagnostics",
    settingsSubtitle: "Configure privacy, LLM enhancement, cache, and exports.",
    llmEnhancement: "LLM Enhancement",
    cache: "Cache",
    exports: "Exports",
    privacySetting: "Local-only mode is enabled. History files never upload unless an LLM action is explicitly run.",
    llmSetting: "Configure ANTHROPIC_API_KEY, base URL, bearer token, and model for digest or summary enhancement.",
    cacheSetting: "Digest cache is invalidated by source mtime and size. Current cache size: 42 MB.",
    exportSetting: "Default format: Prompt. Markdown and JSON remain available per session.",
    commandPlaceholder: "Type a command or search term...",
    searchHistory: "Search history",
    openSessions: "Open sessions",
    buildContext: "Build context pack",
    reviewToday: "Review today work",
    inspectProviders: "Inspect providers",
    openSettings: "Open settings",
  },
  "中文": {
    appTitle: "AI History",
    appSubtitle: "本地记忆监控台",
    navDashboard: "仪表盘",
    navSearch: "搜索",
    navSessions: "会话",
    navContext: "上下文构建",
    navWorklog: "工作日志",
    navWorkflows: "工作流",
    navProviders: "数据源",
    navSettings: "设置",
    collapse: "收起",
    localOnly: "仅本地",
    upToDate: "已是最新",
    threeProviders: "3 个数据源",
    searchPlaceholder: "搜索会话、文件、工具、决策...",
    refresh: "刷新",
    live: "实时",
    dashboardSubtitle: "Claude Code、Codex CLI 和 Cursor 历史记录的本地概览。",
    totalSessions: "总会话数",
    activeToday: "今日 3 个活跃",
    projects: "项目",
    acrossProviders: "来自 3 个数据源",
    messages: "消息",
    indexedLocally: "本地索引",
    todayWork: "今日工作",
    summariesReady: "4 份摘要可用",
    digestCache: "摘要缓存",
    errorSessions: "错误会话",
    hitRate: "命中率",
    privacy: "隐私",
    noUpload: "默认不上传",
    recentSessions: "最近会话",
    viewAll: "查看全部",
    open: "打开",
    filters: "筛选",
    query: "查询",
    searchMode: "搜索模式",
    anyTerms: "任意词匹配",
    allTerms: "全部词匹配",
    contextWindow: "上下文窗口",
    contextWindowHelp: "每个命中前后各 2 条消息",
    results: "条结果",
    relevanceSort: "按相关性排序",
    noResults: "没有匹配会话",
    noResultsHelp: "本机 ai-history CLI 没有返回匹配历史。",
    sampleData: "最近本地会话",
    searchSubtitle: "检索本机 Claude Code、Codex CLI 和 Cursor 历史。",
    prototypeNotice: "真实本地数据模式",
    prototypeNoticeHelp: "仪表盘、会话、数据源、工作日志、工作流和搜索都读取本机 ai-history CLI，不再展示 mock 数据。",
    realSearch: "真实本地检索",
    searchLoading: "正在检索本地 AI 历史...",
    searchError: "本地检索失败",
    searchIdle: "输入关键词后会检索真实本地 AI 历史。未搜索前展示最近本地会话。",
    loadingLocalHistory: "正在加载本地历史...",
    localHistoryError: "本地历史加载失败",
    noLocalSessions: "没有找到本地会话",
    sessionsSubtitle: "浏览项目、检查对话，并生成可复用记忆。",
    export: "导出",
    transcript: "对话",
    digest: "摘要",
    files: "文件",
    tools: "工具",
    intent: "意图",
    keyDecisions: "关键决策",
    codeChanges: "代码变更",
    remainingIssues: "遗留问题",
    copyDigest: "复制摘要",
    enhance: "增强",
    contextSubtitle: "把历史会话转换成下一次 AI 编程可直接使用的上下文。",
    sources: "来源",
    selected: "已选择",
    contextPack: "上下文包",
    output: "输出",
    estimatedTokens: "预计 token",
    copyContext: "复制上下文",
    exportMarkdown: "导出 Markdown",
    worklogSubtitle: "按项目、日期和数据源汇总本地 AI 编程活动。",
    summary: "总结",
    todayWorkTitles: "今日工作标题",
    aiWorkSummary: "AI 工作总结",
    copy: "复制",
    workflowsSubtitle: "识别值得沉淀为 skill 或自动化的重复手动流程。",
    candidates: "候选流程",
    frequency: "频次",
    recommendation: "建议形式",
    coverage: "覆盖情况",
    previewSkill: "预览 Skill 草稿",
    providersSubtitle: "检查本地数据源、扫描状态和诊断信息。",
    connected: "已连接",
    rescan: "重新扫描",
    diagnostics: "诊断",
    settingsSubtitle: "配置隐私、LLM 增强、缓存和导出偏好。",
    llmEnhancement: "LLM 增强",
    cache: "缓存",
    exports: "导出",
    privacySetting: "已启用仅本地模式。除非显式运行 LLM 操作，否则历史文件不会上传。",
    llmSetting: "配置 ANTHROPIC_API_KEY、Base URL、Bearer Token 和摘要增强模型。",
    cacheSetting: "摘要缓存按源文件 mtime 和 size 失效。当前缓存大小：42 MB。",
    exportSetting: "默认格式：Prompt。每个会话仍可导出 Markdown 和 JSON。",
    commandPlaceholder: "输入命令或搜索词...",
    searchHistory: "搜索历史",
    openSessions: "打开会话",
    buildContext: "构建上下文包",
    reviewToday: "查看今日工作",
    inspectProviders: "检查数据源",
    openSettings: "打开设置",
  },
};

const sessions = [
  {
    id: "019e8dc9",
    fullId: "019e8dc9-0add-7020-a320-6054da650131",
    title: "Implement today worklog aggregation",
    provider: "codex",
    project: "/Users/zhangjiantao/Documents/AI/ai-history",
    time: "Today 22:00",
    range: "22:00-22:25",
    messages: 186,
    model: "gpt-5",
    type: "New feature",
    status: "complete",
    tools: ["Bash", "Edit", "Read"],
    files: ["src/today.rs", "src/cli.rs", "README_CN.md"],
    keywords: ["今日", "工作日志", "总结", "摘要", "标题", "跨工具", "本地日期", "json", "provider"],
    summary:
      "Added a cross-provider today command with titles, summary, JSON output, date filters, provider narrowing, and local-date overlap handling.",
    digest: {
      intent: "Create a local daily worklog command for ai-history.",
      decisions: [
        "Use ProviderRegistry::list_all_sessions as the cross-provider entrypoint.",
        "Filter sessions by local-date overlap instead of UTC string matching.",
        "Keep title extraction rule-based and reject prompt scaffolding.",
      ],
      changes: ["src/today.rs", "src/cli.rs", "src/main.rs", "README.md"],
      issues: ["The --summary flag is currently equivalent to default summary output."],
    },
  },
  {
    id: "019e66ab",
    fullId: "019e66ab-e6ff-7180-a9f2-fb80ce6f1abd",
    title: "Mine repeated workflows across AI tools",
    provider: "codex",
    project: "/Users/zhangjiantao/Documents/AI/ai-history",
    time: "May 27",
    range: "07:43-10:06",
    messages: 244,
    model: "gpt-5",
    type: "Workflow",
    status: "complete",
    tools: ["Bash", "Edit", "Read"],
    files: ["src/workflows.rs", "src/provider/codex.rs", "README.md"],
    keywords: ["工作流", "总结", "skill", "技能", "沉淀", "重复流程", "报告", "cursor", "claude", "codex"],
    summary:
      "Added a report-first workflows command that scans Codex, Claude Code, and Cursor before optionally writing selected skill drafts.",
    digest: {
      intent: "Find repeated manual workflows worth packaging as skills.",
      decisions: [
        "Default to report-only suggestions.",
        "Require explicit --write-skills and --skill before writing files.",
        "Scan all supported providers unless the user narrows scope.",
      ],
      changes: ["src/workflows.rs", "src/provider/mod.rs", "README_CN.md"],
      issues: ["Candidate classification is rule-based and intentionally conservative."],
    },
  },
  {
    id: "a247accc",
    fullId: "a247accc-5421-4e48-9d91-2ad70d5828ef",
    title: "Fix OAuth callback regression",
    provider: "claude",
    project: "/Users/jack/myapp",
    time: "May 21",
    range: "09:19-09:31",
    messages: 42,
    model: "Claude Sonnet 4.5",
    type: "Bug fix",
    status: "warning",
    tools: ["Read", "Edit", "Bash"],
    files: ["auth/callback.ts", "tests/auth.test.ts"],
    keywords: ["认证", "登录", "回调", "bug", "修复", "缓存", "测试"],
    summary:
      "Fixed an OAuth redirect loop after callback validation changed and added coverage for stale token cache behavior.",
    digest: {
      intent: "Resolve an OAuth callback loop.",
      decisions: [
        "Keep callback validation inside auth/callback.ts.",
        "Avoid moving token refresh into middleware.",
      ],
      changes: ["auth/callback.ts", "tests/auth.test.ts"],
      issues: ["Manual browser verification still needed."],
    },
  },
  {
    id: "cx7714fb",
    fullId: "cursor-cx7714fb-composer",
    title: "Investigate stale token cache",
    provider: "cursor",
    project: "/Users/jack/myapp",
    time: "May 18",
    range: "14:02-14:47",
    messages: 78,
    model: "agent mode",
    type: "Investigation",
    status: "complete",
    tools: ["agent-mode", "Grep", "Read"],
    files: ["src/cache/token.ts", "src/session.ts"],
    keywords: ["缓存", "token", "会话", "排查", "cursor", "agent", "总结"],
    summary:
      "Traced stale token behavior through the session cache and documented a safer invalidation path.",
    digest: {
      intent: "Understand why token refresh does not invalidate cache.",
      decisions: ["Invalidate by session id, not user id.", "Keep cache TTL unchanged."],
      changes: ["src/cache/token.ts"],
      issues: [],
    },
  },
];

const workflowCandidates = [
  {
    id: "ai-history-workflow-mining",
    name: "Review AI work history for repeatable workflows",
    confidence: "very high",
    frequency: 5,
    coverage: "covered by ai-history-workflow-miner",
    recommendation: "extend existing",
    status: "covered",
  },
  {
    id: "rn-screenshot-code-fix",
    name: "Diagnose React Native visible UI symptoms before editing",
    confidence: "high",
    frequency: 4,
    coverage: "missing",
    recommendation: "skill",
    status: "ready",
  },
  {
    id: "repo-backed-agent-capability-assessment",
    name: "Assess whether a real backend can support AI Agent features",
    confidence: "medium",
    frequency: 2,
    coverage: "covered by repo-backed-agent-capability-assessment",
    recommendation: "extend existing",
    status: "covered",
  },
];

const navItems = [
  { id: "dashboard", labelKey: "navDashboard", icon: LayoutDashboard },
  { id: "search", labelKey: "navSearch", icon: Search },
  { id: "sessions", labelKey: "navSessions", icon: Folder },
  { id: "context", labelKey: "navContext", icon: Brain },
  { id: "worklog", labelKey: "navWorklog", icon: CalendarDays },
  { id: "workflows", labelKey: "navWorkflows", icon: Workflow },
  { id: "providers", labelKey: "navProviders", icon: Database },
  { id: "settings", labelKey: "navSettings", icon: Settings },
];

function ProviderBadge({ provider }) {
  return <span className={`provider provider-${provider}`}>{provider}</span>;
}

function StatusPill({ status, children }) {
  return <span className={`status-pill ${status}`}>{children}</span>;
}

const queryExpansions = {
  总结: ["summary", "digest", "worklog", "workflow", "report"],
  摘要: ["digest", "summary", "context"],
  工作: ["worklog", "today", "workflow"],
  工作流: ["workflow", "skill"],
  会话: ["session", "sessions"],
  搜索: ["search", "bm25"],
  上下文: ["context", "prompt", "digest"],
  缓存: ["cache", "token"],
  工具: ["tool", "tools", "bash", "read", "edit"],
  文件: ["file", "files"],
  修复: ["fix", "bug"],
  认证: ["auth", "oauth", "callback"],
};

function expandQueryTerms(term) {
  const expanded = new Set([term]);
  for (const [key, values] of Object.entries(queryExpansions)) {
    if (term.includes(key) || key.includes(term)) {
      values.forEach((value) => expanded.add(value));
    }
  }
  return Array.from(expanded);
}

function queryTermGroups(query) {
  return query
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
    .map((term) => expandQueryTerms(term));
}

function sessionSearchText(session) {
  return [
    session.title,
    session.summary,
    session.type,
    session.provider,
    session.project,
    session.model,
    session.files.join(" "),
    session.tools.join(" "),
    session.keywords?.join(" "),
    session.digest.intent,
    session.digest.decisions.join(" "),
    session.digest.changes.join(" "),
    session.digest.issues.join(" "),
  ]
    .join(" ")
    .toLowerCase();
}

function MetricCard({ icon: Icon, label, value, meta, tone = "purple" }) {
  return (
    <button className="metric-card" type="button">
      <div>
        <p>{label}</p>
        <strong>{value}</strong>
        {meta && <span>{meta}</span>}
      </div>
      <Icon className={`metric-icon ${tone}`} size={24} />
    </button>
  );
}

function buildContextText(selectedSessions, mode) {
  return selectedSessions
    .map((session) => {
      if (mode === "Full") {
        return `# ${session.title}\nProvider: ${session.provider}\nProject: ${session.project}\n\nUser: ${session.digest.intent}\nAssistant: ${session.summary}`;
      }
      if (mode === "Prompt") {
        return `User: ${session.digest.intent}\nAssistant: ${session.summary}`;
      }
      return `# Session Digest: ${session.title}\n\n## Intent\n${session.digest.intent}\n\n## Key Decisions\n${session.digest.decisions.map((decision) => `- ${decision}`).join("\n")}\n\n## Code Changes\n${session.digest.changes.map((file) => `- Modified ${file}`).join("\n")}`;
    })
    .join("\n\n---\n\n");
}

async function writeClipboard(text) {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    return false;
  }
  return false;
}

function Toast({ toast }) {
  if (!toast) return null;
  return (
    <div className="toast">
      <Check size={18} />
      <span>{toast}</span>
    </div>
  );
}

function PrototypeNotice({ t }) {
  return (
    <div className="prototype-notice">
      <StatusPill status="waiting">{t.prototypeNotice}</StatusPill>
      <span>{t.prototypeNoticeHelp}</span>
    </div>
  );
}

function CommandPalette({ open, onClose, setActive, setGlobalQuery, t }) {
  const [query, setQuery] = useState("");
  const commands = [
    [t.searchHistory, Search, () => setActive("search")],
    [t.openSessions, Folder, () => setActive("sessions")],
    [t.buildContext, Brain, () => setActive("context")],
    [t.reviewToday, CalendarDays, () => setActive("worklog")],
    [t.inspectProviders, Database, () => setActive("providers")],
    [t.openSettings, Settings, () => setActive("settings")],
  ].filter(([label]) => label.toLowerCase().includes(query.toLowerCase()));

  if (!open) return null;
  return (
    <div className="modal-backdrop command-backdrop" onClick={onClose}>
      <div className="command-palette" onClick={(event) => event.stopPropagation()}>
        <div className="command-input">
          <Search size={20} />
          <input
            autoFocus
            placeholder={t.commandPlaceholder}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && query.trim()) {
                setGlobalQuery(query.trim());
                setActive("search");
                onClose();
              }
              if (event.key === "Escape") {
                onClose();
              }
            }}
          />
        </div>
        <div className="command-list">
          {commands.map(([label, Icon, action]) => (
            <button
              key={label}
              type="button"
              onClick={() => {
                action();
                onClose();
              }}
            >
              <Icon size={18} />
              <span>{label}</span>
              <ChevronRight size={16} />
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function Sidebar({ active, setActive, t }) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark">
          <History size={22} />
        </div>
        <div>
          <h1>{t.appTitle}</h1>
          <p>{t.appSubtitle}</p>
        </div>
      </div>

      <nav className="nav-list">
        {navItems.map((item) => {
          const Icon = item.icon;
          return (
            <button
              key={item.id}
              className={active === item.id ? "nav-item active" : "nav-item"}
              onClick={() => setActive(item.id)}
              type="button"
            >
              <Icon size={20} />
              <span>{t[item.labelKey]}</span>
            </button>
          );
        })}
      </nav>

      <div className="sidebar-spacer" />

      <button className="collapse-button" type="button">
        <PanelLeftClose size={18} />
        <span>{t.collapse}</span>
      </button>

      <div className="system-status">
        <div>
          <ShieldCheck size={18} />
          <span>{t.localOnly}</span>
          <small>v0.3.0</small>
        </div>
        <div>
          <RefreshCcw size={18} />
          <span>{t.upToDate}</span>
        </div>
        <div>
          <Archive size={18} />
          <span>{t.threeProviders}</span>
        </div>
      </div>
    </aside>
  );
}

function TopBar({
  activeProvider,
  setActiveProvider,
  language,
  setLanguage,
  globalQuery,
  setGlobalQuery,
  setActive,
  openCommands,
  onRefresh,
  refreshed,
  t,
}) {
  return (
    <div className="topbar">
      <div className="global-search">
        <Search size={18} />
        <input
          placeholder={t.searchPlaceholder}
          value={globalQuery}
          onChange={(event) => setGlobalQuery(event.target.value)}
          onFocus={() => setActive("search")}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              setActive("search");
            }
          }}
        />
        <button className="kbd-button" onClick={openCommands} type="button">⌘K</button>
      </div>
      <div className="topbar-actions">
        <div className="segmented provider-switch">
          {providers.map((provider) => (
            <button
              key={provider.id}
              className={activeProvider === provider.id ? "active" : ""}
              onClick={() => setActiveProvider(provider.id)}
              type="button"
            >
              {provider.label}
            </button>
          ))}
        </div>
        <div className="segmented language-switch" aria-label="Language">
          {["EN", "中文"].map((item) => (
            <button
              key={item}
              className={language === item ? "active" : ""}
              onClick={() => setLanguage(item)}
              type="button"
            >
              {item}
            </button>
          ))}
        </div>
        <button className="ghost-button" onClick={onRefresh} type="button">
          <RefreshCcw size={18} className={refreshed ? "spin-once" : ""} />
          {t.refresh}
        </button>
      </div>
    </div>
  );
}

function Dashboard({ filteredSessions, metrics, todayEntries, loading, error, setActive, setSelectedSession, t }) {
  const latest = filteredSessions.slice(0, 3);
  return (
    <section className="screen">
      <div className="screen-title">
        <div className="title-icon"><LayoutDashboard size={25} /></div>
        <div>
          <h2>{t.navDashboard} <StatusPill status="live">{t.live}</StatusPill></h2>
          <p>{t.dashboardSubtitle}</p>
        </div>
      </div>

      {loading && <div className="search-state-card"><RefreshCcw className="spin-once" size={20} /><span>{t.loadingLocalHistory}</span></div>}
      {error && <div className="search-state-card error"><TerminalSquare size={20} /><span>{t.localHistoryError}: {error}</span></div>}

      <div className="metrics-grid">
        <MetricCard icon={Folder} label={t.totalSessions} value={metrics.totalSessions || 0} meta={t.indexedLocally} />
        <MetricCard icon={Database} label={t.projects} value={metrics.projects || 0} meta={`${metrics.providers || 0} providers`} tone="blue" />
        <MetricCard icon={MessageSquareText} label={t.messages} value={metrics.messages || 0} meta={t.indexedLocally} tone="green" />
        <MetricCard icon={CalendarDays} label={t.todayWork} value={metrics.todayWork || 0} meta={t.summariesReady} tone="yellow" />
        <MetricCard icon={Brain} label={t.errorSessions} value={metrics.errorSessions || 0} meta={t.remainingIssues} tone="purple" />
        <MetricCard icon={ShieldCheck} label={t.privacy} value={t.localOnly} meta={t.noUpload} tone="green" />
      </div>

      <div className="dashboard-grid">
        <div className="panel large">
          <div className="panel-header">
            <h3>{t.recentSessions}</h3>
            <button onClick={() => setActive("sessions")} type="button">{t.viewAll} <ChevronRight size={16} /></button>
          </div>
          <div className="session-stack">
            {latest.length === 0 && (
              <div className="empty-state">
                <Folder size={28} />
                <strong>{t.noLocalSessions}</strong>
                <p>{t.noResultsHelp}</p>
              </div>
            )}
            {latest.map((session) => (
              <button
                className="session-card"
                key={session.fullId}
                onClick={() => {
                  setSelectedSession(session);
                  setActive("sessions");
                }}
                type="button"
              >
                <div className="session-icon"><Bot size={20} /></div>
                <div className="session-main">
                  <div className="session-line">
                    <strong>{session.title}</strong>
                    <StatusPill status={session.status === "warning" ? "waiting" : "complete"}>{session.status}</StatusPill>
                  </div>
                  <p>{session.summary}</p>
                  <div className="meta-row">
                    <span><Clock size={15} /> {session.range}</span>
                    <span><MessageSquareText size={15} /> {session.messages} msgs</span>
                    <ProviderBadge provider={session.provider} />
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>

        <div className="panel activity-panel">
          <div className="panel-header">
            <h3>{t.todayWork}</h3>
            <button onClick={() => setActive("worklog")} type="button">{t.open} <ChevronRight size={16} /></button>
          </div>
          <div className="activity-list">
            {todayEntries.length === 0 && (
              <div className="empty-state">
                <CalendarDays size={28} />
                <strong>{t.noLocalSessions}</strong>
                <p>{t.noResultsHelp}</p>
              </div>
            )}
            {todayEntries.slice(0, 5).map((entry) => (
              <div className="activity-item" key={entry.fullId || entry.sessionId}>
                <StatusPill status="complete">complete</StatusPill>
                <span>{entry.title}</span>
                <small>{entry.provider}</small>
                <em>{entry.range}</em>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function SearchScreen({
  filteredSessions,
  selectedSession,
  setSelectedSession,
  query,
  setQuery,
  activeProvider,
  notify,
  t,
}) {
  const [mode, setMode] = useState("any");
  const [realSearchState, setRealSearchState] = useState({
    status: "idle",
    results: [],
    error: "",
  });
  const hasRealQuery = query.trim().length > 0;

  useEffect(() => {
    const q = query.trim();
    if (!q) {
      setRealSearchState({ status: "idle", results: [], error: "" });
      return undefined;
    }

    const controller = new AbortController();
    const timeout = window.setTimeout(async () => {
      setRealSearchState((current) => ({ ...current, status: "loading", error: "" }));
      try {
        const params = new URLSearchParams({
          q,
          mode,
          provider: activeProvider,
          limit: "20",
          context: "2",
        });
        const response = await fetch(`/api/search?${params.toString()}`, {
          signal: controller.signal,
        });
        const payload = await response.json();
        if (!response.ok) {
          throw new Error(payload.error || "Search request failed");
        }
        setRealSearchState({
          status: "success",
          results: payload.results || [],
          error: "",
        });
      } catch (error) {
        if (error.name === "AbortError") return;
        setRealSearchState({
          status: "error",
          results: [],
          error: error.message,
        });
      }
    }, 280);

    return () => {
      controller.abort();
      window.clearTimeout(timeout);
    };
  }, [activeProvider, mode, query]);

  const sampleResults = filteredSessions.filter((session) => {
    const q = query.trim().toLowerCase();
    if (!q) return true;
    const termGroups = queryTermGroups(q);
    const haystack = sessionSearchText(session);
    if (mode === "all") {
      return termGroups.every((group) => group.some((term) => haystack.includes(term)));
    }
    return (
      haystack.includes(q) ||
      termGroups.some((group) => group.some((term) => haystack.includes(term)))
    );
  });
  const results = hasRealQuery ? realSearchState.results : sampleResults;
  const previewSession = results.find((session) => session.fullId === selectedSession?.fullId) || results[0];

  return (
    <section className="screen">
      <div className="screen-title">
        <div className="title-icon"><Search size={25} /></div>
        <div>
          <h2>{t.navSearch}</h2>
          <p>{t.searchSubtitle}</p>
        </div>
      </div>

      <div className="three-column">
        <aside className="filter-panel panel">
          <h3>{t.filters}</h3>
          <label className="field">
            <span>{t.query}</span>
            <input value={query} onChange={(event) => setQuery(event.target.value)} />
          </label>
          <label className="field">
            <span>{t.searchMode}</span>
            <select value={mode} onChange={(event) => setMode(event.target.value)}>
              <option value="any">{t.anyTerms}</option>
              <option value="all">{t.allTerms}</option>
            </select>
          </label>
          <div className="check-list">
            {["User", "Assistant", "Tool", "System"].map((item, index) => (
              <label key={item}>
                <input type="checkbox" defaultChecked={index < 3} />
                <span>{item}</span>
              </label>
            ))}
          </div>
          <div className="range-card">
            <ListFilter size={18} />
            <div>
              <strong>{t.contextWindow}</strong>
              <p>{t.contextWindowHelp}</p>
            </div>
          </div>
        </aside>

        <div className="panel results-panel">
          <div className="panel-header">
            <h3>{results.length} {t.results}</h3>
            <span className="muted">
              {hasRealQuery ? t.realSearch : t.sampleData} · {mode === "all" ? t.allTerms : t.anyTerms}
            </span>
          </div>
          <div className="result-list">
            {!hasRealQuery && (
              <div className="search-state-card">
                <Search size={20} />
                <span>{t.searchIdle}</span>
              </div>
            )}
            {realSearchState.status === "loading" && (
              <div className="search-state-card">
                <RefreshCcw className="spin-once" size={20} />
                <span>{t.searchLoading}</span>
              </div>
            )}
            {realSearchState.status === "error" && (
              <div className="search-state-card error">
                <TerminalSquare size={20} />
                <span>{t.searchError}: {realSearchState.error}</span>
              </div>
            )}
            {results.length === 0 && realSearchState.status !== "loading" && realSearchState.status !== "error" && (
              <div className="empty-state">
                <Search size={28} />
                <strong>{t.noResults}</strong>
                <p>{t.noResultsHelp}</p>
                <div className="chip-row">
                  {["总结", "工作流", "缓存", "auth"].map((preset) => (
                    <button key={preset} className="chip-button" onClick={() => setQuery(preset)} type="button">
                      {preset}
                    </button>
                  ))}
                </div>
              </div>
            )}
            {results.map((session, index) => (
              <button
                key={`${session.fullId}-${index}`}
                className={selectedSession?.fullId === session.fullId ? "result-card selected" : "result-card"}
              onClick={() => setSelectedSession(session)}
              type="button"
            >
                <div className="score">{(session.score ?? (8.9 - index * 1.2)).toFixed(1)}</div>
                <div>
                  <div className="session-line">
                    <strong>{session.title}</strong>
                    <ProviderBadge provider={session.provider} />
                  </div>
                  <p>{session.summary}</p>
                  <div className="meta-row">
                    <span>{session.project}</span>
                    <span>{session.time}</span>
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>

        {previewSession ? (
          <SessionPreview
            session={previewSession}
            compact
            notify={notify}
            t={t}
          />
        ) : (
          <div className="panel preview-panel compact empty-preview">
            <ProviderBadge provider="all" />
            <h3>{t.sampleData}</h3>
            <p>{t.noResultsHelp}</p>
          </div>
        )}
      </div>
    </section>
  );
}

function SessionsScreen({ filteredSessions, projects, selectedSession, setSelectedSession, notify, t }) {
  const visibleSelectedSession =
    filteredSessions.find((session) => session.fullId === selectedSession?.fullId) || filteredSessions[0];
  return (
    <section className="screen">
      <div className="screen-title">
        <div className="title-icon"><Folder size={25} /></div>
        <div>
          <h2>{t.navSessions}</h2>
          <p>{t.sessionsSubtitle}</p>
        </div>
      </div>

      <div className="sessions-layout">
        <aside className="panel project-list">
          <div className="panel-header">
            <h3>{t.projects}</h3>
            <small>{projects.length}</small>
          </div>
          {projects.map((project) => (
            <button className="project-row active" key={`${project.provider}-${project.name}`} type="button">
              <Folder size={18} />
              <span>{project.name}</span>
              <small>{project.session_count}</small>
            </button>
          ))}
        </aside>

        <div className="panel session-browser">
          <div className="panel-header">
            <h3>{t.navSessions}</h3>
            <button onClick={() => notify("Sessions exported as JSON")} type="button"><Download size={16} /> {t.export}</button>
          </div>
          {filteredSessions.length === 0 && (
            <div className="empty-state">
              <Folder size={28} />
              <strong>{t.noLocalSessions}</strong>
              <p>{t.noResultsHelp}</p>
            </div>
          )}
          {filteredSessions.map((session) => (
            <button
              key={session.fullId}
              className={selectedSession?.fullId === session.fullId ? "browser-row selected" : "browser-row"}
              onClick={() => setSelectedSession(session)}
              type="button"
            >
              <div className="session-icon"><FileText size={18} /></div>
              <div>
                <strong>{session.title}</strong>
                <p>{session.summary}</p>
                <div className="meta-row">
                  <ProviderBadge provider={session.provider} />
                  <span>{session.messages} messages</span>
                  <span>{session.time}</span>
                </div>
              </div>
            </button>
          ))}
        </div>

        {visibleSelectedSession ? (
          <SessionPreview session={visibleSelectedSession} notify={notify} t={t} />
        ) : (
          <div className="panel preview-panel empty-preview">
            <h3>{t.noLocalSessions}</h3>
            <p>{t.noResultsHelp}</p>
          </div>
        )}
      </div>
    </section>
  );
}

function SessionPreview({ session, compact = false, notify = () => {}, t }) {
  const [tab, setTab] = useState("digest");
  const tabs = [
    ["transcript", t.transcript],
    ["digest", t.digest],
    ["files", t.files],
    ["tools", t.tools],
  ];

  return (
    <aside className={compact ? "panel preview-panel compact" : "panel preview-panel"}>
      <div className="preview-title">
        <ProviderBadge provider={session.provider} />
        <h3>{session.title}</h3>
        <p>{session.fullId}</p>
      </div>
      <div className="tab-strip">
        {tabs.map(([id, label]) => (
          <button
            key={id}
            className={tab === id ? "active" : ""}
            onClick={() => setTab(id)}
            type="button"
          >
            {label}
          </button>
        ))}
      </div>

      {tab === "digest" && (
        <div className="digest-view">
          <section>
            <h4>{t.intent}</h4>
            <p>{session.digest.intent}</p>
          </section>
          <section>
            <h4>{t.keyDecisions}</h4>
            {session.digest.decisions.map((decision) => <p key={decision}>- {decision}</p>)}
          </section>
          <section>
            <h4>{t.codeChanges}</h4>
            <div className="chip-row">
              {session.digest.changes.map((file) => <span className="chip" key={file}>{file}</span>)}
            </div>
          </section>
          {session.digest.issues.length > 0 && (
            <section>
              <h4>{t.remainingIssues}</h4>
              {session.digest.issues.map((issue) => <p key={issue}>- {issue}</p>)}
            </section>
          )}
          <div className="action-row">
            <button
              className="primary-button"
              onClick={async () => {
                await writeClipboard(buildContextText([session], "Digest"));
                notify("Digest copied");
              }}
              type="button"
            >
              <Copy size={17} /> {t.copyDigest}
            </button>
            <button onClick={() => notify("LLM enhancement preview generated")} type="button"><Sparkles size={17} /> {t.enhance}</button>
          </div>
        </div>
      )}

      {tab === "transcript" && (
        <div className="transcript">
          {(session.transcript || [
            { role: "User", text: session.digest.intent },
            { role: "Assistant", text: session.summary },
            { role: "Tool · Bash", text: "cargo test · finished with existing warnings" },
            { role: "Assistant", text: "Implemented and verified the scoped change." },
          ]).map((message, index) => (
            <div className={`message ${message.role?.toLowerCase() || "assistant"}`} key={`${message.timestamp || index}-${message.role}`}>
              <strong>{message.toolName ? `${message.role} · ${message.toolName}` : message.role}</strong>
              <p>{message.text}</p>
            </div>
          ))}
        </div>
      )}

      {tab === "files" && (
        <div className="file-list">
          {session.files.map((file) => (
            <div className="file-row" key={file}>
              <FileCode2 size={17} />
              <span>{file}</span>
              <small>modified</small>
            </div>
          ))}
        </div>
      )}

      {tab === "tools" && (
        <div className="tool-bars">
          {session.tools.map((tool, index) => (
            <div className="tool-bar" key={tool}>
              <span>{tool}</span>
              <div><i style={{ width: `${92 - index * 18}%` }} /></div>
              <small>{12 - index * 3}</small>
            </div>
          ))}
        </div>
      )}
    </aside>
  );
}

function ContextBuilder({ availableSessions, selectedSession, notify, t }) {
  const [mode, setMode] = useState("Digest");
  const defaultIds = useMemo(() => availableSessions.slice(0, 2).map((session) => session.fullId), [availableSessions]);
  const [include, setInclude] = useState([]);
  useEffect(() => {
    setInclude(defaultIds);
  }, [defaultIds]);
  const selected = availableSessions.filter((session) => include.includes(session.fullId));
  const tokenEstimate = mode === "Full" ? "18.4K" : mode === "Prompt" ? "7.8K" : "2.1K";

  function toggleSession(fullId) {
    setInclude((current) => current.includes(fullId) ? current.filter((item) => item !== fullId) : [...current, fullId]);
  }

  return (
    <section className="screen">
      <div className="screen-title">
        <div className="title-icon"><Brain size={25} /></div>
        <div>
          <h2>{t.navContext}</h2>
          <p>{t.contextSubtitle}</p>
        </div>
      </div>

      <div className="builder-layout">
        <aside className="panel source-panel">
          <div className="panel-header">
            <h3>{t.sources}</h3>
            <small>{include.length} {t.selected}</small>
          </div>
          {availableSessions.length === 0 && (
            <div className="empty-state">
              <Brain size={28} />
              <strong>{t.noLocalSessions}</strong>
              <p>{t.noResultsHelp}</p>
            </div>
          )}
          {availableSessions.map((session) => (
            <label className="source-row" key={session.fullId}>
              <input
                checked={include.includes(session.fullId)}
                onChange={() => toggleSession(session.fullId)}
                type="checkbox"
              />
              <div>
                <strong>{session.title}</strong>
                <p>{session.provider} · {session.id}</p>
              </div>
            </label>
          ))}
        </aside>

        <main className="panel context-editor">
          <div className="panel-header">
            <h3>{t.contextPack}</h3>
            <div className="segmented small">
              {["Digest", "Prompt", "Full"].map((item) => (
                <button
                  className={mode === item ? "active" : ""}
                  onClick={() => setMode(item)}
                  type="button"
                  key={item}
                >
                  {item}
                </button>
              ))}
            </div>
          </div>

          <div className="pack-list">
            {selected.map((session, index) => (
              <div className="pack-card" key={session.fullId}>
                <span>{index + 1}</span>
                <div>
                  <strong>{session.title}</strong>
                  <p>{session.digest.intent}</p>
                  <div className="chip-row">
                    {session.files.slice(0, 2).map((file) => <span className="chip" key={file}>{file}</span>)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </main>

        <aside className="panel output-panel">
          <h3>{t.output}</h3>
          <div className="token-meter">
            <Gauge size={22} />
            <div>
              <strong>{tokenEstimate}</strong>
              <p>{t.estimatedTokens}</p>
            </div>
          </div>
          <div className="preview-box">
            <h4># Context Pack</h4>
            <p>Includes {selected.length} sessions, formatted as {mode.toLowerCase()} context.</p>
            <p>Primary seed: {selectedSession?.title || t.noLocalSessions}</p>
          </div>
          <button
            className="primary-button full"
            onClick={async () => {
              await writeClipboard(buildContextText(selected, mode));
              notify(`${mode} context copied`);
            }}
            type="button"
          >
            <Clipboard size={17} /> {t.copyContext}
          </button>
          <button onClick={() => notify("Markdown export prepared")} className="full" type="button"><Download size={17} /> {t.exportMarkdown}</button>
        </aside>
      </div>
    </section>
  );
}

function Worklog({ todayEntries, notify, t }) {
  const [tab, setTab] = useState("today");
  return (
    <section className="screen">
      <div className="screen-title">
        <div className="title-icon"><CalendarDays size={25} /></div>
        <div>
          <h2>{t.navWorklog}</h2>
          <p>{t.worklogSubtitle}</p>
        </div>
      </div>
      <div className="tab-strip wide">
        {[["today", t.todayWork], ["summary", t.summary]].map(([id, label]) => (
          <button key={id} className={tab === id ? "active" : ""} onClick={() => setTab(id)} type="button">{label}</button>
        ))}
      </div>
      <div className="panel">
        <div className="panel-header">
          <h3>{tab === "today" ? t.todayWorkTitles : t.aiWorkSummary}</h3>
          <button onClick={() => notify(`${tab} worklog copied`)} type="button"><Copy size={16} /> {t.copy}</button>
        </div>
        <div className="worklog-table">
          {todayEntries.length === 0 && (
            <div className="empty-state">
              <CalendarDays size={28} />
              <strong>{t.noLocalSessions}</strong>
              <p>{t.noResultsHelp}</p>
            </div>
          )}
          {todayEntries.map((entry) => (
            <div className="table-row" key={entry.fullId || entry.sessionId}>
              <span>{entry.range}</span>
              <strong>{entry.title}</strong>
              <ProviderBadge provider={entry.provider} />
              <span>{entry.project}</span>
              <small>{entry.messages} msgs</small>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function WorkflowsScreen({ activeProvider, notify, t }) {
  const [workflowState, setWorkflowState] = useState({ status: "loading", candidates: [], error: "" });
  const [selected, setSelected] = useState(null);
  const [modal, setModal] = useState(false);

  useEffect(() => {
    const controller = new AbortController();
    async function loadWorkflows() {
      setWorkflowState({ status: "loading", candidates: [], error: "" });
      try {
        const params = new URLSearchParams({ provider: activeProvider });
        const response = await fetch(`/api/workflows?${params.toString()}`, { signal: controller.signal });
        const payload = await response.json();
        if (!response.ok) throw new Error(payload.error || "Workflow scan failed");
        const candidates = payload.report?.candidates || [];
        setWorkflowState({ status: "success", candidates, error: "" });
        setSelected(candidates[0] || null);
      } catch (error) {
        if (error.name === "AbortError") return;
        setWorkflowState({ status: "error", candidates: [], error: error.message });
        setSelected(null);
      }
    }
    loadWorkflows();
    return () => controller.abort();
  }, [activeProvider]);

  return (
    <section className="screen">
      <div className="screen-title">
        <div className="title-icon"><Workflow size={25} /></div>
        <div>
          <h2>{t.navWorkflows}</h2>
          <p>{t.workflowsSubtitle}</p>
        </div>
      </div>
      <div className="workflow-layout">
        <div className="panel">
          <div className="panel-header">
            <h3>{t.candidates}</h3>
            <span className="muted">last 30 days · min 2 sessions</span>
          </div>
          {workflowState.status === "loading" && (
            <div className="search-state-card">
              <RefreshCcw className="spin-once" size={20} />
              <span>{t.loadingLocalHistory}</span>
            </div>
          )}
          {workflowState.status === "error" && (
            <div className="search-state-card error">
              <TerminalSquare size={20} />
              <span>{t.localHistoryError}: {workflowState.error}</span>
            </div>
          )}
          {workflowState.status === "success" && workflowState.candidates.length === 0 && (
            <div className="empty-state">
              <Workflow size={28} />
              <strong>{t.noLocalSessions}</strong>
              <p>{t.noResultsHelp}</p>
            </div>
          )}
          {workflowState.candidates.map((candidate) => (
            <button
              key={candidate.id}
              className={selected?.id === candidate.id ? "workflow-row selected" : "workflow-row"}
              onClick={() => setSelected(candidate)}
              type="button"
            >
              <div>
                <strong>{candidate.workflow}</strong>
                <p>{candidate.coverage}</p>
              </div>
              <StatusPill status={candidate.worth_creating ? "working" : "complete"}>
                {candidate.confidence}
              </StatusPill>
            </button>
          ))}
        </div>
        <aside className="panel workflow-detail">
          {selected ? (
            <>
              <h3>{selected.id}</h3>
              <p>{selected.workflow}</p>
              <div className="detail-grid">
                <div><span>{t.frequency}</span><strong>{selected.frequency}</strong></div>
                <div><span>{t.recommendation}</span><strong>{selected.recommended_form}</strong></div>
                <div><span>{t.coverage}</span><strong>{selected.coverage}</strong></div>
              </div>
              <div className="evidence-list">
                {(selected.evidence || []).map((item) => (
                  <div key={`${item.session_id}-${item.date}`}>
                    <Clock size={15} />
                    <span>{item.date}</span>
                    <strong>{item.summary}</strong>
                  </div>
                ))}
              </div>
              <button
                className="primary-button full"
                disabled={!selected.worth_creating}
                onClick={() => setModal(true)}
                type="button"
              >
                <FileText size={17} />
                {t.previewSkill}
              </button>
            </>
          ) : (
            <div className="empty-state">
              <Workflow size={28} />
              <strong>{t.noLocalSessions}</strong>
              <p>{t.noResultsHelp}</p>
            </div>
          )}
        </aside>
      </div>
      {modal && selected && (
        <div className="modal-backdrop" onClick={() => setModal(false)}>
          <div className="modal" onClick={(event) => event.stopPropagation()}>
            <h3>Skill Draft Preview</h3>
            <p>{selected.suggested_skill_name || selected.id}</p>
            <pre>{selected.rationale}</pre>
            <div className="action-row">
              <button onClick={() => setModal(false)} type="button">Cancel</button>
              <button
                className="primary-button"
                onClick={() => {
                  setModal(false);
                  notify("Use ai-history workflows --write-skills --skill to write this real candidate");
                }}
                type="button"
              >
                <Check size={17} /> Write Skill
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}

function ProvidersScreen({ projects, notify, t }) {
  const [diagnostic, setDiagnostic] = useState(null);
  const items = ["claude", "codex", "cursor"].map((provider) => {
    const providerProjects = projects.filter((project) => project.provider === provider);
    const sessions = providerProjects.reduce((sum, project) => sum + (project.session_count || 0), 0);
    const latest = providerProjects[0]?.last_modified || "";
    const name = provider === "claude" ? "Claude Code" : provider === "codex" ? "Codex CLI" : "Cursor";
    const path = provider === "claude" ? "~/.claude/projects" : provider === "codex" ? "~/.codex/sessions" : "~/Library/Application Support/Cursor/User";
    return { name, path, status: providerProjects.length ? t.connected : t.noLocalSessions, count: `${sessions} sessions`, detail: `${providerProjects.length} projects · latest ${latest}` };
  });
  return (
    <section className="screen">
      <div className="screen-title">
        <div className="title-icon"><Database size={25} /></div>
        <div>
          <h2>{t.navProviders}</h2>
          <p>{t.providersSubtitle}</p>
        </div>
      </div>
      <div className="provider-grid">
        {items.map(({ name, path, status, count, detail }) => (
          <div className="panel provider-card" key={name}>
            <div className="provider-head">
              <HardDrive size={24} />
              <StatusPill status="working">{status}</StatusPill>
            </div>
            <h3>{name}</h3>
            <p>{path}</p>
            <strong>{count}</strong>
            <div className="action-row">
              <button onClick={() => notify(`${name} rescan complete`)} type="button"><RefreshCcw size={16} /> {t.rescan}</button>
              <button onClick={() => setDiagnostic({ name, path, detail })} type="button"><TerminalSquare size={16} /> {t.diagnostics}</button>
            </div>
          </div>
        ))}
      </div>
      {diagnostic && (
        <div className="modal-backdrop" onClick={() => setDiagnostic(null)}>
          <div className="modal" onClick={(event) => event.stopPropagation()}>
            <h3>{diagnostic.name} Diagnostics</h3>
            <p>{diagnostic.path}</p>
            <pre>{`status: connected\npath: ${diagnostic.path}\nscan: ok\nnotes: ${diagnostic.detail}`}</pre>
            <div className="action-row">
              <button onClick={() => setDiagnostic(null)} type="button">Close</button>
              <button
                className="primary-button"
                onClick={() => {
                  notify("Diagnostics copied");
                  setDiagnostic(null);
                }}
                type="button"
              >
                <Copy size={17} /> Copy Diagnostics
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}

function SettingsScreen({ notify, t }) {
  return (
    <section className="screen">
      <div className="screen-title">
        <div className="title-icon"><Settings size={25} /></div>
        <div>
          <h2>{t.navSettings}</h2>
          <p>{t.settingsSubtitle}</p>
        </div>
      </div>
      <div className="settings-grid">
        {[
          [t.privacy, ShieldCheck, t.privacySetting],
          [t.llmEnhancement, KeyRound, t.llmSetting],
          [t.cache, Box, t.cacheSetting],
          [t.exports, Download, t.exportSetting],
        ].map(([title, Icon, text]) => (
          <div className="panel setting-card" key={title}>
            <Icon size={23} />
            <div>
              <h3>{title}</h3>
              <p>{text}</p>
            </div>
            <label className="toggle">
              <input
                type="checkbox"
                defaultChecked
                onChange={(event) => notify(`${title} ${event.target.checked ? "enabled" : "disabled"}`)}
              />
              <span />
            </label>
          </div>
        ))}
      </div>
    </section>
  );
}

export function App() {
  const [active, setActive] = useState("dashboard");
  const [activeProvider, setActiveProvider] = useState("all");
  const [selectedSession, setSelectedSession] = useState(null);
  const [refreshed, setRefreshed] = useState(false);
  const [language, setLanguage] = useState("中文");
  const [globalQuery, setGlobalQuery] = useState("");
  const [toast, setToast] = useState("");
  const [commandsOpen, setCommandsOpen] = useState(false);
  const [refreshToken, setRefreshToken] = useState(0);
  const [historyState, setHistoryState] = useState({
    status: "loading",
    data: {
      projects: [],
      recentSessions: [],
      todayEntries: [],
      metrics: {},
    },
    error: "",
  });
  const t = i18n[language];

  useEffect(() => {
    const controller = new AbortController();
    async function loadDashboardData() {
      setHistoryState((current) => ({ ...current, status: "loading", error: "" }));
      try {
        const params = new URLSearchParams({ provider: activeProvider });
        const response = await fetch(`/api/dashboard?${params.toString()}`, { signal: controller.signal });
        const payload = await response.json();
        if (!response.ok) throw new Error(payload.error || "Dashboard load failed");
        setHistoryState({
          status: "success",
          data: {
            projects: payload.projects || [],
            recentSessions: payload.recentSessions || [],
            todayEntries: payload.todayEntries || [],
            metrics: payload.metrics || {},
          },
          error: "",
        });
      } catch (error) {
        if (error.name === "AbortError") return;
        setHistoryState((current) => ({ ...current, status: "error", error: error.message }));
      }
    }
    loadDashboardData();
    return () => controller.abort();
  }, [activeProvider, refreshToken]);

  const projects = historyState.data.projects;
  const filteredSessions = historyState.data.recentSessions;
  const todayEntries = historyState.data.todayEntries;
  const metrics = historyState.data.metrics;

  useEffect(() => {
    setSelectedSession((current) => {
      if (current && filteredSessions.some((session) => session.fullId === current.fullId)) return current;
      return filteredSessions[0] || null;
    });
  }, [filteredSessions]);

  function handleRefresh() {
    setRefreshed(true);
    setRefreshToken((current) => current + 1);
    notify("Local history rescan complete");
    setTimeout(() => setRefreshed(false), 700);
  }

  function notify(message) {
    setToast(message);
    setTimeout(() => setToast(""), 2200);
  }

  useEffect(() => {
    function handleKeyDown(event) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandsOpen(true);
      }
      if (event.key === "Escape") {
        setCommandsOpen(false);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <div className="app-shell">
      <Sidebar active={active} setActive={setActive} t={t} />
      <main className="main-area">
        <TopBar
          activeProvider={activeProvider}
          setActiveProvider={setActiveProvider}
          language={language}
          setLanguage={setLanguage}
          globalQuery={globalQuery}
          setGlobalQuery={setGlobalQuery}
          setActive={setActive}
          openCommands={() => setCommandsOpen(true)}
          onRefresh={handleRefresh}
          refreshed={refreshed}
          t={t}
        />
        <PrototypeNotice t={t} />
        {active === "dashboard" && (
          <Dashboard
            filteredSessions={filteredSessions}
            metrics={metrics}
            todayEntries={todayEntries}
            loading={historyState.status === "loading"}
            error={historyState.error}
            setActive={setActive}
            setSelectedSession={setSelectedSession}
            t={t}
          />
        )}
        {active === "search" && (
          <SearchScreen
            filteredSessions={filteredSessions}
            selectedSession={selectedSession}
            setSelectedSession={setSelectedSession}
            query={globalQuery}
            setQuery={setGlobalQuery}
            activeProvider={activeProvider}
            notify={notify}
            t={t}
          />
        )}
        {active === "sessions" && (
          <SessionsScreen
            filteredSessions={filteredSessions}
            projects={projects}
            selectedSession={selectedSession}
            setSelectedSession={setSelectedSession}
            notify={notify}
            t={t}
          />
        )}
        {active === "context" && (
          <ContextBuilder
            availableSessions={filteredSessions}
            selectedSession={selectedSession}
            notify={notify}
            t={t}
          />
        )}
        {active === "worklog" && <Worklog todayEntries={todayEntries} notify={notify} t={t} />}
        {active === "workflows" && <WorkflowsScreen activeProvider={activeProvider} notify={notify} t={t} />}
        {active === "providers" && <ProvidersScreen projects={projects} notify={notify} t={t} />}
        {active === "settings" && <SettingsScreen notify={notify} t={t} />}
      </main>
      <CommandPalette
        open={commandsOpen}
        onClose={() => setCommandsOpen(false)}
        setActive={setActive}
        setGlobalQuery={setGlobalQuery}
        t={t}
      />
      <Toast toast={toast} />
    </div>
  );
}
