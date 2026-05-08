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
        /// Search query
        query: String,

        /// Max results
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
    },

    /// Export a session
    Export {
        /// Session ID or path fragment
        session: String,

        /// Export format
        #[arg(short, long, default_value = "md")]
        format: ExportFormat,
    },
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Md,
    Json,
    Prompt,
}
