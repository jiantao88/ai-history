import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const aiHistoryBinary = process.env.AI_HISTORY_BIN || `${process.env.HOME}/.cargo/bin/ai-history`;
const currentProject = process.env.AI_HISTORY_PROJECT || process.cwd().replace(/\/prototypes\/ai-history-mac$/, "");

function sendJson(res, statusCode, payload) {
  res.statusCode = statusCode;
  res.setHeader("Content-Type", "application/json");
  res.end(JSON.stringify(payload));
}

async function runAiHistory(args, options = {}) {
  const { stdout } = await execFileAsync(aiHistoryBinary, args, {
    timeout: options.timeout || 60000,
    maxBuffer: options.maxBuffer || 1024 * 1024 * 16,
  });
  return JSON.parse(stdout || "[]");
}

function truncateText(text, length = 220) {
  if (!text) return "";
  return text.length > length ? `${text.slice(0, length)}...` : text;
}

function basename(path) {
  return (path || "").split("/").filter(Boolean).pop() || path || "local project";
}

function cleanHistoryText(text) {
  return (text || "")
    .replace(/# AGENTS\.md instructions for[^\n]*(\n|$)/gi, "")
    .replace(/<INSTRUCTIONS>[\s\S]*?(<\/INSTRUCTIONS>|$)/gi, "")
    .replace(/<INSTRUCT[^>\s]*(\s|>|$)[\s\S]*/gi, "")
    .replace(/<!--\s*CODEGRAPH_START\s*-->[\s\S]*?(<!--\s*CODEGRAPH_END\s*-->|$)/gi, "")
    .replace(/<environment_context>[\s\S]*?(<\/environment_context>|$)/gi, "")
    .replace(/<command-message>[\s\S]*?(<\/command-message>|$)/gi, "")
    .replace(/<command-name>[\s\S]*?(<\/command-name>|$)/gi, "")
    .replace(/<command-args>|<\/command-args>/gi, "")
    .replace(/--- project-doc ---[\s\S]*/gi, "")
    .replace(/\s+/g, " ")
    .trim();
}

function createDisplayTitle(session, fallbackText = "") {
  const cleaned = cleanHistoryText(session.summary || fallbackText);
  if (cleaned && cleaned.length > 8 && !/^<|^#?\s*AGENTS\.md/i.test(cleaned)) {
    return truncateText(cleaned, 96);
  }
  return `${session.provider} session ${session.id.slice(0, 8)} · ${basename(session.project_name)}`;
}

function createDisplaySummary(session, fallbackText = "") {
  const cleaned = cleanHistoryText(session.summary || fallbackText);
  if (cleaned && cleaned.length > 8 && !/^<|^#?\s*AGENTS\.md/i.test(cleaned)) {
    return truncateText(cleaned, 180);
  }
  const tools = session.metadata?.tools_used?.length ? `工具：${session.metadata.tools_used.join(", ")}` : "本地历史会话";
  return `${basename(session.project_name)} · ${tools}`;
}

function formatTimeRange(firstTime, lastTime) {
  const first = firstTime ? new Date(firstTime) : null;
  const last = lastTime ? new Date(lastTime) : null;
  if (!first || Number.isNaN(first.getTime())) return lastTime || "";
  const start = first.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
  if (!last || Number.isNaN(last.getTime())) return start;
  const end = last.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
  return `${start}-${end}`;
}

function createSessionFromCli(session) {
  const metadata = session.metadata || {};
  const files = metadata.files_touched || [];
  const tools = metadata.tools_used || [];
  const title = createDisplayTitle(session);
  const summary = createDisplaySummary(session);
  return {
    id: session.id.slice(0, 8),
    fullId: session.id,
    title,
    provider: session.provider,
    project: session.project_name,
    time: session.last_time || session.first_time || "",
    range: formatTimeRange(session.first_time, session.last_time),
    messages: session.message_count || 0,
    model: "local history",
    type: metadata.has_errors ? "error-context" : "session",
    status: metadata.has_errors ? "warning" : "complete",
    tools: tools.length ? tools : ["ai-history"],
    files,
    keywords: [],
    summary,
    digest: {
      intent: summary,
      decisions: tools.length ? [`使用工具：${tools.join(", ")}`] : [],
      changes: files,
      issues: metadata.has_errors ? ["包含错误排查或失败输出"] : [],
    },
    transcript: [],
    isRealResult: true,
  };
}

function providerArgs(provider) {
  return provider && provider !== "all" ? ["--provider", provider] : [];
}

async function getProjects(provider) {
  const args = ["list", "--json", ...providerArgs(provider)];
  const projects = await runAiHistory(args);
  return projects.sort((a, b) => new Date(b.last_modified || 0) - new Date(a.last_modified || 0));
}

async function getSessionsForProject(project, provider) {
  const effectiveProvider = provider === "all" ? project.provider : provider;
  const args = ["sessions", project.name || project.path, "--json", ...providerArgs(effectiveProvider)];
  const sessions = await runAiHistory(args, { timeout: 45000 });
  return sessions.map(createSessionFromCli);
}

async function getRecentSessions(projects, provider, limit = 30) {
  const candidates = projects.slice(0, 14);
  const batches = await Promise.allSettled(candidates.map((project) => getSessionsForProject(project, provider)));
  return batches
    .flatMap((result) => result.status === "fulfilled" ? result.value : [])
    .sort((a, b) => new Date(b.time || 0) - new Date(a.time || 0))
    .slice(0, limit);
}

async function getTodayEntries(provider) {
  try {
    const args = ["today", currentProject, "--json", "--all-providers", ...providerArgs(provider)];
    const entries = await runAiHistory(args, { timeout: 45000 });
    return entries.map((entry) => ({
      title: entry.title,
      provider: entry.provider,
      sessionId: entry.session_id,
      fullId: entry.session_id_full,
      project: entry.project,
      firstTime: entry.first_time,
      lastTime: entry.last_time,
      range: formatTimeRange(entry.first_time, entry.last_time),
      messages: entry.message_count || 0,
      files: entry.files_touched || [],
      summary: Array.isArray(entry.summary) ? entry.summary : [],
    }));
  } catch {
    return [];
  }
}

async function getWorkflowReport(provider) {
  const args = ["workflows", "--json", ...providerArgs(provider)];
  return runAiHistory(args, { timeout: 90000, maxBuffer: 1024 * 1024 * 24 });
}

function createSearchSession(result) {
  const messageText = result.message?.text || "";
  const before = result.context_before || [];
  const after = result.context_after || [];
  const contextMessages = [...before, result.message, ...after].filter(Boolean);
  const tools = Array.from(
    new Set(
      contextMessages
        .map((message) => message.tool_name)
        .filter(Boolean)
    )
  );
  const model = contextMessages.find((message) => message.model)?.model || "local history";
  const textSnippet = truncateText(cleanHistoryText(messageText) || messageText, 180);
  const sessionLike = {
    id: result.session_id,
    provider: result.provider,
    project_name: result.project_name,
    summary: textSnippet,
  };

  return {
    id: result.session_id.slice(0, 8),
    fullId: result.session_id,
    title: createDisplayTitle(sessionLike, textSnippet || `${result.provider} search result`),
    provider: result.provider,
    project: result.project_name,
    time: result.message?.timestamp || "",
    range: result.message?.timestamp || "",
    messages: contextMessages.length || 1,
    model,
    type: result.message?.role || "match",
    status: "complete",
    tools: tools.length ? tools : ["ai-history"],
    files: [],
    keywords: [],
    summary: textSnippet || "Matched local AI history.",
    score: result.score,
    matchIndex: result.match_index,
    digest: {
      intent: textSnippet || "Matched local AI history.",
      decisions: after.slice(0, 3).map((message) => message.text).filter(Boolean),
      changes: [],
      issues: [],
    },
    transcript: contextMessages.map((message) => ({
      role: message.role,
      text: message.text,
      toolName: message.tool_name,
      timestamp: message.timestamp,
    })),
    isRealResult: true,
  };
}

function aiHistoryApiPlugin() {
  return {
    name: "ai-history-api",
    configureServer(server) {
      server.middlewares.use("/api/dashboard", async (req, res) => {
        try {
          const url = new URL(req.url || "", "http://127.0.0.1");
          const provider = url.searchParams.get("provider") || "all";
          const projects = await getProjects(provider);
          const recentSessions = await getRecentSessions(projects, provider);
          const todayEntries = await getTodayEntries(provider);
          const totalSessions = projects.reduce((sum, project) => sum + (project.session_count || 0), 0);
          const messageTotal = recentSessions.reduce((sum, session) => sum + (session.messages || 0), 0);
          const providerSet = new Set(projects.map((project) => project.provider));

          sendJson(res, 200, {
            source: "real",
            provider,
            currentProject,
            projects,
            recentSessions,
            todayEntries,
            metrics: {
              totalSessions,
              projects: projects.length,
              providers: providerSet.size,
              messages: messageTotal,
              todayWork: todayEntries.length,
              errorSessions: recentSessions.filter((session) => session.status === "warning").length,
            },
          });
        } catch (error) {
          sendJson(res, 500, {
            source: "real",
            error: error.message,
            stderr: error.stderr,
          });
        }
      });

      server.middlewares.use("/api/workflows", async (req, res) => {
        try {
          const url = new URL(req.url || "", "http://127.0.0.1");
          const provider = url.searchParams.get("provider") || "all";
          const report = await getWorkflowReport(provider);
          sendJson(res, 200, { source: "real", provider, report });
        } catch (error) {
          sendJson(res, 500, {
            source: "real",
            error: error.message,
            stderr: error.stderr,
          });
        }
      });

      server.middlewares.use("/api/search", async (req, res) => {
        try {
          const url = new URL(req.url || "", "http://127.0.0.1");
          const query = url.searchParams.get("q")?.trim() || "";
          const provider = url.searchParams.get("provider") || "all";
          const mode = url.searchParams.get("mode") || "any";
          const limit = Math.min(Number(url.searchParams.get("limit") || 20), 50);
          const context = Math.min(Number(url.searchParams.get("context") || 2), 5);

          if (!query) {
            sendJson(res, 200, { source: "real", query, results: [] });
            return;
          }

          const args = [
            "search",
            query,
            "--limit",
            String(limit),
            "--context",
            String(context),
            "--json",
          ];

          if (mode === "all") {
            args.push("--all");
          }
          if (provider !== "all") {
            args.push("--provider", provider);
          }

          const { stdout } = await execFileAsync(aiHistoryBinary, args, {
            timeout: 45000,
            maxBuffer: 1024 * 1024 * 8,
          });
          const rawResults = JSON.parse(stdout || "[]");
          sendJson(res, 200, {
            source: "real",
            query,
            provider,
            mode,
            results: rawResults.map(createSearchSession),
          });
        } catch (error) {
          sendJson(res, 500, {
            source: "real",
            error: error.message,
            stderr: error.stderr,
          });
        }
      });
    },
  };
}

export default defineConfig({
  base: "./",
  optimizeDeps: {
    include: ["react", "react-dom/client"],
  },
  server: {
    warmup: {
      clientFiles: ["./src/main.jsx"],
    },
  },
  plugins: [aiHistoryApiPlugin(), react()],
});
