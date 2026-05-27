use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Result};
use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::model::Session;
use crate::provider::ProviderRegistry;

#[derive(Debug)]
pub struct WorkflowOptions<'a> {
    pub project_filter: Option<&'a str>,
    pub days: i64,
    pub range: Option<&'a str>,
    pub min_sessions: usize,
    pub include_subagents: bool,
    pub write_skills: bool,
    pub selected_skill_ids: Vec<String>,
    pub skills_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowReport {
    pub date_label: String,
    pub total_sessions_reviewed: usize,
    pub candidates: Vec<WorkflowCandidate>,
    pub written_skills: Vec<WrittenSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCandidate {
    pub id: String,
    pub workflow: String,
    pub evidence: Vec<WorkflowEvidence>,
    pub frequency: usize,
    pub confidence: String,
    pub recommended_form: String,
    pub coverage: String,
    pub worth_creating: bool,
    pub rationale: String,
    pub suggested_skill_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvidence {
    pub date: String,
    pub provider: String,
    pub project: String,
    pub session_id: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrittenSkill {
    pub candidate_id: String,
    pub skill_name: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WorkflowKind {
    AiHistoryWorkflowMining,
    RnScreenshotCodeFix,
    RepoBackedAgentAssessment,
    CrossToolCodeMap,
    RnBatchRefactorValidation,
    ProductPrdPrototype,
    InterviewBackendPrimer,
    RemoteSshDiagnosis,
    CodeReviewFixCycle,
}

#[derive(Debug, Clone)]
struct SessionSignal {
    kind: WorkflowKind,
    evidence: WorkflowEvidence,
}

pub fn build_workflow_report(
    registry: &ProviderRegistry,
    opts: &WorkflowOptions<'_>,
    provider_filter: Option<&[String]>,
) -> Result<WorkflowReport> {
    if opts.days <= 0 && opts.range.is_none() {
        bail!("--days must be greater than 0");
    }
    if opts.write_skills && opts.selected_skill_ids.is_empty() {
        bail!("Use --skill <candidate-id> to choose which skill drafts to write");
    }

    let date_range = parse_workflow_date_range(opts.days, opts.range)?;
    let date_label = format_date_label(&date_range);
    let existing_skills = discover_existing_skills();

    let mut signals = Vec::new();
    let mut total_sessions_reviewed = 0;

    let sessions = registry.list_all_sessions(provider_filter)?;
    for session in sessions {
        if !opts.include_subagents && session.is_subagent {
            continue;
        }
        if let Some(project_filter) = opts.project_filter {
            if !session.project_name.contains(project_filter)
                && !session.file_path.contains(project_filter)
            {
                continue;
            }
        }
        if !session_in_range(&session, &date_range) || session.message_count <= 1 {
            continue;
        }

        total_sessions_reviewed += 1;

        let text = session_text_for_classification(&session);
        let Some(kind) = classify_workflow(&session.project_name, &text) else {
            continue;
        };

        signals.push(SessionSignal {
            kind,
            evidence: WorkflowEvidence {
                date: session_date_label(&session),
                provider: session.provider.clone(),
                project: session.project_name.clone(),
                session_id: short_id(&session.id),
                summary: summarize_session(&session),
            },
        });
    }

    if provider_allows_codex(provider_filter) {
        signals.extend(scan_codex_rollout_summaries(
            &date_range,
            opts.project_filter,
        )?);
    }

    let mut grouped: BTreeMap<String, (WorkflowKind, Vec<WorkflowEvidence>)> = BTreeMap::new();
    for signal in signals {
        let spec = workflow_spec(signal.kind);
        grouped
            .entry(spec.id.to_string())
            .or_insert_with(|| (signal.kind, Vec::new()))
            .1
            .push(signal.evidence);
    }

    let mut candidates = Vec::new();
    for (id, (kind, mut evidence)) in grouped {
        evidence.sort_by(|a, b| a.date.cmp(&b.date));
        evidence.dedup_by(|a, b| a.session_id == b.session_id && a.provider == b.provider);

        if evidence.len() < opts.min_sessions {
            continue;
        }

        let spec = workflow_spec(kind);
        let coverage = coverage_for(spec, &existing_skills);
        let covered = coverage != "missing";
        let worth_creating = !covered && spec.recommended_form == "skill";

        candidates.push(WorkflowCandidate {
            id,
            workflow: spec.workflow.to_string(),
            frequency: evidence.len(),
            confidence: confidence_for(evidence.len()),
            recommended_form: if covered {
                "extend existing".to_string()
            } else {
                spec.recommended_form.to_string()
            },
            coverage,
            worth_creating,
            rationale: if covered {
                spec.covered_rationale.to_string()
            } else {
                spec.rationale.to_string()
            },
            suggested_skill_name: spec.skill_name.map(str::to_string),
            evidence,
        });
    }

    candidates.sort_by(|a, b| {
        b.worth_creating
            .cmp(&a.worth_creating)
            .then(b.frequency.cmp(&a.frequency))
            .then(a.id.cmp(&b.id))
    });

    let written_skills = if opts.write_skills {
        write_selected_skills(&candidates, opts)?
    } else {
        Vec::new()
    };

    Ok(WorkflowReport {
        date_label,
        total_sessions_reviewed,
        candidates,
        written_skills,
    })
}

#[derive(Debug, Clone, Copy)]
struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

fn parse_workflow_date_range(days: i64, range: Option<&str>) -> Result<DateRange> {
    if let Some(range_str) = range {
        let parts: Vec<&str> = range_str.split("..").collect();
        if parts.len() != 2 {
            bail!("Invalid range format. Use YYYY-MM-DD..YYYY-MM-DD");
        }
        let start = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid start date: {}", parts[0]))?;
        let end = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid end date: {}", parts[1]))?;
        if end < start {
            bail!("Range end must be on or after start");
        }
        return Ok(DateRange { start, end });
    }

    let end = Local::now().date_naive();
    let start = end - Duration::days(days.saturating_sub(1));
    Ok(DateRange { start, end })
}

fn format_date_label(range: &DateRange) -> String {
    if range.start == range.end {
        range.start.format("%Y-%m-%d").to_string()
    } else {
        format!(
            "{} ~ {}",
            range.start.format("%Y-%m-%d"),
            range.end.format("%Y-%m-%d")
        )
    }
}

fn session_in_range(session: &Session, range: &DateRange) -> bool {
    session_date(session)
        .map(|d| d >= range.start && d <= range.end)
        .unwrap_or(false)
}

fn session_date(session: &Session) -> Option<NaiveDate> {
    chrono::DateTime::parse_from_rfc3339(&session.first_time)
        .ok()
        .map(|dt| dt.with_timezone(&Local).date_naive())
}

fn session_date_label(session: &Session) -> String {
    session_date(session)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

fn session_text_for_classification(session: &Session) -> String {
    let mut parts = Vec::new();
    parts.push(session.project_name.clone());
    if let Some(summary) = &session.summary {
        parts.push(summary.clone());
    }
    if let Some(meta) = &session.metadata {
        parts.extend(meta.files_touched.iter().take(20).cloned());
        parts.extend(meta.tools_used.iter().take(20).cloned());
        parts.extend(meta.languages.iter().cloned());
    }
    parts.join("\n").to_lowercase()
}

fn summarize_session(session: &Session) -> String {
    if let Some(summary) = session.summary.as_deref() {
        let cleaned = clean_summary(summary);
        if !cleaned.is_empty() && !cleaned.starts_with("<environment_context>") {
            return truncate_chars(&cleaned, 90);
        }
    }
    "-".to_string()
}

fn clean_summary(text: &str) -> String {
    strip_xml_tags(text)
        .replace("[Image #1]", "")
        .replace("[Image #2]", "")
        .replace("[Image #3]", "")
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('/'))
        .unwrap_or("")
        .to_string()
}

fn strip_xml_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}...",
            s.chars().take(max.saturating_sub(3)).collect::<String>()
        )
    }
}

fn classify_workflow(project_name: &str, text: &str) -> Option<WorkflowKind> {
    let project = project_name.to_lowercase();
    let is_rn = project.contains("rnproject")
        || text.contains("react native")
        || text.contains(".tsx")
        || text.contains("rn ");

    if has_any(
        text,
        &[
            "开发周报",
            "工作回顾",
            "最近工作",
            "重复手动工作",
            "workflow mining",
        ],
    ) {
        return Some(WorkflowKind::AiHistoryWorkflowMining);
    }

    if has_any(
        text,
        &[
            "速查表",
            "code map",
            "codemap",
            "quick reference",
            "quick-reference",
            "代码定位速查表",
            "ai_codemap",
        ],
    ) {
        return Some(WorkflowKind::CrossToolCodeMap);
    }

    if is_rn
        && has_any(
            text,
            &[
                "截图",
                "image #",
                "样式",
                "显示",
                "字段",
                "接口",
                "手势",
                "滑",
                "点击",
                "前端",
                "后端",
                "缓存",
                "gesture",
                "scrollview",
                "carousel",
                "image preview",
                "preview page",
            ],
        )
        && has_any(
            text,
            &[
                "修复",
                "问题",
                "取的",
                "来源",
                "哪个",
                "为什么",
                "改回",
                "误",
                "fix",
                "replaced",
            ],
        )
    {
        return Some(WorkflowKind::RnScreenshotCodeFix);
    }

    let agent_focused = has_any(
        text,
        &[
            "ai agent",
            "agent service",
            "agent 架构",
            "agent 场景",
            "预测建议",
            "智能体",
            "agent capability",
            "agent suitability",
        ],
    ) || (text.contains("agent")
        && has_any(text, &["数据库", "适合", "预测", "mindbet"]));
    if agent_focused
        && has_any(
            text,
            &[
                "当前",
                "项目",
                "后端",
                "repo",
                "service",
                "mindbet",
                "social-im",
                "支持",
            ],
        )
    {
        return Some(WorkflowKind::RepoBackedAgentAssessment);
    }

    if is_rn
        && has_any(
            text,
            &[
                "批量",
                "batch",
                "重构",
                "scrollview",
                "touchableopacity",
                "pressable",
                "提交代码并继续",
            ],
        )
    {
        return Some(WorkflowKind::RnBatchRefactorValidation);
    }

    if has_any(
        text,
        &[
            "prd",
            "原型",
            "prototype",
            "需求文档",
            "产品原型",
            "reviewable",
        ],
    ) && has_any(text, &["skill", "pm-", "需求", "运营后台"])
    {
        return Some(WorkflowKind::ProductPrdPrototype);
    }

    if has_any(
        text,
        &["面试", "候选", "郑亚凯", "单中旭", "hr", "评语", "二面"],
    ) && has_any(text, &["讲解", "评语", "二面", "知识", "后端", "问题"])
    {
        return Some(WorkflowKind::InterviewBackendPrimer);
    }

    if has_any(
        text,
        &[
            "ssh",
            "remote",
            "远程",
            "服务器",
            "publickey",
            "authorized_keys",
            "codex",
        ],
    ) && has_any(text, &["permission denied", "连接", "安装", "诊断", "访问"])
    {
        return Some(WorkflowKind::RemoteSshDiagnosis);
    }

    if has_any(text, &["review", "codereview", "代码审查", "/review"])
        && has_any(text, &["修复", "fix", "再次", "发现"])
    {
        return Some(WorkflowKind::CodeReviewFixCycle);
    }

    None
}

fn has_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn provider_allows_codex(provider_filter: Option<&[String]>) -> bool {
    provider_filter
        .map(|providers| providers.iter().any(|p| p == "codex"))
        .unwrap_or(true)
}

fn scan_codex_rollout_summaries(
    range: &DateRange,
    project_filter: Option<&str>,
) -> Result<Vec<SessionSignal>> {
    let Some(home) = dirs::home_dir() else {
        return Ok(Vec::new());
    };
    let dir = home.join(".codex/memories/rollout_summaries");
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(Vec::new());
    };

    let mut signals = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(date) = file_name
            .get(0..10)
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        else {
            continue;
        };
        if date < range.start || date > range.end {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap_or_default();
        let cwd = extract_frontmatter_value(&content, "cwd").unwrap_or_default();
        if let Some(filter) = project_filter {
            if !cwd.contains(filter) && !content.contains(filter) {
                continue;
            }
        }

        let title = extract_markdown_title(&content)
            .unwrap_or_else(|| file_name.trim_end_matches(".md").to_string());
        let opening: String = content.chars().take(1500).collect();
        let classification_text =
            format!("{}\n{}\n{}\n{}", cwd, file_name, title, opening).to_lowercase();
        let Some(kind) = classify_workflow(&cwd, &classification_text) else {
            continue;
        };

        signals.push(SessionSignal {
            kind,
            evidence: WorkflowEvidence {
                date: date.format("%Y-%m-%d").to_string(),
                provider: "codex-memory".to_string(),
                project: if cwd.is_empty() { "-".to_string() } else { cwd },
                session_id: extract_frontmatter_value(&content, "thread_id")
                    .map(|id| short_id(&id))
                    .unwrap_or_else(|| file_name.chars().take(18).collect()),
                summary: title,
            },
        });
    }

    Ok(signals)
}

fn extract_frontmatter_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    content.lines().find_map(|line| {
        line.strip_prefix(&prefix)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    })
}

fn extract_markdown_title(content: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|v| !v.is_empty())
        .map(|v| truncate_chars(v, 90))
}

#[derive(Debug, Clone, Copy)]
struct WorkflowSpec {
    id: &'static str,
    workflow: &'static str,
    recommended_form: &'static str,
    skill_name: Option<&'static str>,
    existing_skill_names: &'static [&'static str],
    rationale: &'static str,
    covered_rationale: &'static str,
}

fn workflow_spec(kind: WorkflowKind) -> WorkflowSpec {
    match kind {
        WorkflowKind::AiHistoryWorkflowMining => WorkflowSpec {
            id: "ai-history-workflow-mining",
            workflow: "Review recent AI work history and identify repeated manual workflows worth packaging",
            recommended_form: "skill",
            skill_name: Some("ai-history-workflow-miner"),
            existing_skill_names: &["ai-history-workflow-miner"],
            rationale: "This is context-heavy, repeatable, and benefits from a fixed evidence order and packaging gates.",
            covered_rationale: "A close workflow-mining skill already exists; reuse or extend it instead of creating another.",
        },
        WorkflowKind::RnScreenshotCodeFix => WorkflowSpec {
            id: "rn-screenshot-code-fix",
            workflow: "Diagnose React Native visible UI symptoms, screenshots, fields, gestures, and cache issues before making a scoped fix",
            recommended_form: "skill",
            skill_name: Some("rn-screenshot-code-fix"),
            existing_skill_names: &["rn-screenshot-code-fix", "react-native-best-practices"],
            rationale: "The workflow repeats across screenshots and UI symptoms, and quality depends on tracing the real code path before editing.",
            covered_rationale: "A related RN workflow skill already exists; extend it if the current one is too broad.",
        },
        WorkflowKind::RepoBackedAgentAssessment => WorkflowSpec {
            id: "repo-backed-agent-capability-assessment",
            workflow: "Assess whether a real repo/backend can support an AI Agent, recommendation, or analysis feature",
            recommended_form: "skill",
            skill_name: Some("repo-backed-agent-capability-assessment"),
            existing_skill_names: &["repo-backed-agent-capability-assessment"],
            rationale: "The task recurs across products and needs a stable repo-first procedure to avoid generic architecture advice.",
            covered_rationale: "A repo-backed Agent assessment skill already exists; reuse or extend it.",
        },
        WorkflowKind::CrossToolCodeMap => WorkflowSpec {
            id: "cross-tool-code-location-doc",
            workflow: "Build a shared code-location quick reference for feature descriptions or screenshots",
            recommended_form: "extend existing",
            skill_name: Some("cross-tool-code-location-doc"),
            existing_skill_names: &["cross-tool-code-location-doc"],
            rationale: "This is already a clear repeatable workflow and should be handled by the existing skill.",
            covered_rationale: "Already covered by the cross-tool code-location doc skill.",
        },
        WorkflowKind::RnBatchRefactorValidation => WorkflowSpec {
            id: "rnproject-batch-refactor-validation",
            workflow: "Validate and commit RN refactor batches with targeted checks and Chinese docs",
            recommended_form: "extend existing",
            skill_name: Some("rnproject-batch-refactor-validation"),
            existing_skill_names: &["rnproject-batch-refactor-validation"],
            rationale: "This is a repeatable high-risk workflow, but it is already covered.",
            covered_rationale: "Already covered by the rnproject batch validation skill.",
        },
        WorkflowKind::ProductPrdPrototype => WorkflowSpec {
            id: "product-prd-to-prototype",
            workflow: "Convert raw product material into PRD and reviewable prototype",
            recommended_form: "extend existing",
            skill_name: None,
            existing_skill_names: &["pm-prd-writer", "pm-image2proto", "space-image2proto", "web-prototype"],
            rationale: "The flow is repeatable, but existing PM/prototype skills likely cover it.",
            covered_rationale: "Existing PRD and prototype skills cover most of this workflow.",
        },
        WorkflowKind::InterviewBackendPrimer => WorkflowSpec {
            id: "backend-interview-concept-primer",
            workflow: "Turn backend interview/candidate material into plain-language explanations and follow-up questions",
            recommended_form: "skip",
            skill_name: None,
            existing_skill_names: &[],
            rationale: "The evidence is sensitive and candidate-specific; wait for a narrower repeatable boundary before packaging.",
            covered_rationale: "Skipped because the workflow is sensitive and not yet narrow enough.",
        },
        WorkflowKind::RemoteSshDiagnosis => WorkflowSpec {
            id: "codex-remote-ssh-diagnosis",
            workflow: "Diagnose Codex remote SSH and server service visibility issues",
            recommended_form: "skip",
            skill_name: None,
            existing_skill_names: &[],
            rationale: "Useful but not frequent enough yet; keep as memory until another similar case appears.",
            covered_rationale: "Skipped until there is more evidence of recurrence.",
        },
        WorkflowKind::CodeReviewFixCycle => WorkflowSpec {
            id: "review-fix-cycle",
            workflow: "Run review-first code inspection, then implement and verify findings",
            recommended_form: "skip",
            skill_name: None,
            existing_skill_names: &[],
            rationale: "The assistant already has review behavior instructions; a separate skill would likely overlap.",
            covered_rationale: "Covered by general review behavior rather than a dedicated skill.",
        },
    }
}

fn confidence_for(count: usize) -> String {
    match count {
        0 | 1 => "low".to_string(),
        2 => "medium".to_string(),
        3..=4 => "high".to_string(),
        _ => "very high".to_string(),
    }
}

fn discover_existing_skills() -> HashMap<String, PathBuf> {
    let mut skills = HashMap::new();
    let mut roots = Vec::new();
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join(".codex/skills"));
        roots.push(home.join(".codex/memories/skills"));
        roots.push(home.join(".claude/skills"));
    }

    for root in roots {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path().join("SKILL.md");
            if path.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    skills.insert(name.to_string(), path);
                }
            }
        }
    }

    skills
}

fn coverage_for(spec: WorkflowSpec, existing_skills: &HashMap<String, PathBuf>) -> String {
    for name in spec.existing_skill_names {
        if existing_skills.contains_key(*name) {
            return format!("covered by {name}");
        }
    }
    "missing".to_string()
}

fn write_selected_skills(
    candidates: &[WorkflowCandidate],
    opts: &WorkflowOptions<'_>,
) -> Result<Vec<WrittenSkill>> {
    let selected: Vec<_> = opts
        .selected_skill_ids
        .iter()
        .map(|id| {
            candidates
                .iter()
                .find(|c| &c.id == id)
                .ok_or_else(|| anyhow::anyhow!("Unknown workflow candidate: {id}"))
        })
        .collect::<Result<Vec<_>>>()?;

    let base_dir = opts
        .skills_dir
        .clone()
        .or_else(|| dirs::home_dir().map(|home| home.join(".codex/skills")))
        .unwrap_or_else(|| PathBuf::from("skills"));

    let mut written = Vec::new();
    for candidate in selected {
        if !candidate.worth_creating {
            bail!(
                "Candidate '{}' is not marked worth_creating; refusing to write overlapping or skipped skill",
                candidate.id
            );
        }
        let Some(skill_name) = candidate.suggested_skill_name.as_deref() else {
            bail!("Candidate '{}' has no suggested skill name", candidate.id);
        };

        let skill_dir = base_dir.join(skill_name);
        fs::create_dir_all(&skill_dir)?;
        let skill_path = skill_dir.join("SKILL.md");
        if skill_path.exists() {
            bail!("Skill already exists: {}", skill_path.display());
        }

        fs::write(&skill_path, render_skill_draft(candidate, skill_name))?;
        written.push(WrittenSkill {
            candidate_id: candidate.id.clone(),
            skill_name: skill_name.to_string(),
            path: skill_path.display().to_string(),
        });
    }

    Ok(written)
}

fn render_skill_draft(candidate: &WorkflowCandidate, skill_name: &str) -> String {
    let evidence = candidate
        .evidence
        .iter()
        .take(5)
        .map(|e| {
            format!(
                "- {} {} {}: {}",
                e.date, e.provider, e.session_id, e.summary
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "---\nname: {skill_name}\ndescription: {description}\n---\n\n# {skill_name}\n\nUse this skill when the user asks for this repeated workflow:\n\n{workflow}\n\n## Evidence\n\n{evidence}\n\n## Procedure\n\n1. Confirm the user's exact goal and input source.\n2. Gather the relevant local evidence before making recommendations or edits.\n3. Follow the repo's existing patterns and keep the work narrowly scoped.\n4. Produce the expected output with clear verification or stop conditions.\n5. State what was verified, what was skipped, and what needs more evidence.\n\n## Packaging note\n\nThis draft was generated by `ai-history workflows`. Review and tighten it before relying on it broadly.\n",
        skill_name = skill_name,
        description = yaml_escape(&candidate.workflow),
        workflow = candidate.workflow,
        evidence = evidence,
    )
}

fn yaml_escape(s: &str) -> String {
    format!("{:?}", s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_rn_screenshot_fix() {
        let text = "活动列表里 x人参与 是从哪个字段取的？截图显示样式有问题，需要修复";
        assert_eq!(
            classify_workflow("/Users/me/rnproject", text),
            Some(WorkflowKind::RnScreenshotCodeFix)
        );
    }

    #[test]
    fn classifies_agent_assessment() {
        let text = "当前后端项目用的是哪个数据库，适合 Agent 吗，能否支持预测建议";
        assert_eq!(
            classify_workflow("/Users/me/social-im", text),
            Some(WorkflowKind::RepoBackedAgentAssessment)
        );
    }

    #[test]
    fn parses_days_range() {
        let range = parse_workflow_date_range(30, Some("2026-04-27..2026-05-27")).unwrap();
        assert_eq!(range.start.to_string(), "2026-04-27");
        assert_eq!(range.end.to_string(), "2026-05-27");
    }
}
