use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "ai-history", about = "Search and export AI coding assistant chat history")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Filter to specific provider(s), comma-separated
    #[arg(long, global = true, value_delimiter = ',')]
    pub provider: Option<Vec<String>>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all projects across providers
    List,

    /// List sessions in a project
    Sessions {
        /// Project name (fuzzy matched)
        project: String,

        /// Exclude subagent sessions
        #[arg(long)]
        no_subagents: bool,
    },

    /// Display a session's messages
    Show {
        /// Session ID or path fragment
        session: String,

        /// Show only user/assistant messages
        #[arg(long)]
        compact: bool,
    },

    /// Search across all history
    Search {
        /// Search query (supports multiple terms)
        query: String,

        /// Max results
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,

        /// Show N messages before and after each match
        #[arg(short = 'C', long = "context", default_value = "0")]
        context_window: usize,

        /// Require all query terms to match (AND mode)
        #[arg(long = "all")]
        require_all: bool,

        /// Sort by time instead of relevance
        #[arg(long = "sort-time")]
        sort_by_time: bool,
    },

    /// Export a session
    Export {
        /// Session ID or path fragment
        session: String,

        /// Export format
        #[arg(short, long, default_value = "md")]
        format: ExportFormat,
    },

    /// Export a session as context for another AI session (digest by default)
    Context {
        /// Session ID or path fragment
        session: String,

        /// Output full conversation instead of digest
        #[arg(long)]
        full: bool,

        /// Use Claude API for enhanced digest (requires ANTHROPIC_API_KEY)
        #[arg(long)]
        llm: bool,
    },

    /// Summarize AI work for a time period
    Summary {
        /// Project name (fuzzy matched); omit to summarize all projects
        project: Option<String>,

        /// Summarize a specific date (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,

        /// Summarize a date range (YYYY-MM-DD..YYYY-MM-DD)
        #[arg(long)]
        range: Option<String>,

        /// Summarize today (default when no date/range given)
        #[arg(long)]
        today: bool,

        /// Use Claude API for enhanced summary (requires ANTHROPIC_API_KEY)
        #[arg(long)]
        ai_summary: bool,
    },

    /// Generate a session digest (compressed summary)
    Digest {
        /// Session ID or path fragment
        session: String,

        /// Use Claude API for enhanced narrative (requires ANTHROPIC_API_KEY)
        #[arg(long)]
        llm: bool,

        /// Skip cache and regenerate
        #[arg(long)]
        no_cache: bool,
    },
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Md,
    Json,
    Prompt,
}
