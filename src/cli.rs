//! The command-line interface definition.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "producthunt",
    version,
    about = "Fast native CLI for searching and scraping Product Hunt launches and products via Apify",
    after_help = "An LLM agent should start with: producthunt agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Search Product Hunt for products and launches
    Search(SearchArgs),
    /// Scrape a Product Hunt URL (product page, launch, topic, or collection)
    Scrape(ScrapeArgs),
    /// Get current authenticated user context from Apify
    Me(AuthArgs),
    /// Log in with an Apify API token (opens browser to copy token)
    Login(LoginArgs),
    /// Manage Apify accounts and stored tokens
    #[command(subcommand)]
    Accounts(AccountsCommand),
    /// Print the operating manual for an LLM agent driving this CLI
    AgentReadme,
}

#[derive(Args, Clone, Debug)]
pub struct SearchArgs {
    /// Search query (product name, category, or keyword)
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// Maximum results to return
    #[arg(long, default_value = "10", value_name = "N")]
    pub max: usize,

    /// Sort by: popular, newest
    #[arg(long, default_value = "popular", value_name = "SORT")]
    pub sort: String,

    /// Optional time frame filter: today, this-week, this-month, all-time
    #[arg(long, value_name = "TIMEFRAME")]
    pub time_frame: Option<String>,

    /// Account to run against (see 'producthunt accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct ScrapeArgs {
    /// Product Hunt URL to scrape (product page, launch, topic, or collection)
    #[arg(value_name = "URL")]
    pub url: String,

    /// Maximum results to return
    #[arg(long, default_value = "10", value_name = "N")]
    pub max: usize,

    /// Account to run against (see 'producthunt accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct AuthArgs {
    /// Account to run against (see 'producthunt accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct LoginArgs {
    /// Account name to store (default: "default")
    #[arg(value_name = "NAME", default_value = "default")]
    pub name: String,

    /// Apify API token (prompted for securely if omitted)
    #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
    pub api_key: Option<String>,

    /// Read the API token from stdin
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not open the browser to the API tokens page automatically
    #[arg(long)]
    pub no_browser: bool,

    /// Replace the token on an account that already exists
    #[arg(long)]
    pub force: bool,

    /// Store the token without calling the API to verify it first
    #[arg(long)]
    pub no_verify: bool,
}

#[derive(Subcommand, Clone, Debug)]
pub enum AccountsCommand {
    /// Add an account and store its API token in the OS keystore
    Add {
        /// Short name for this account, used as --account elsewhere
        name: String,
        /// Apify API token (prompted for without echo if omitted)
        #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
        api_key: Option<String>,
        /// Read the API token from stdin
        #[arg(long)]
        api_key_stdin: bool,
        /// Replace the token on an account that already exists
        #[arg(long)]
        force: bool,
        /// Store the token without calling the API to verify it first
        #[arg(long)]
        no_verify: bool,
    },
    /// List configured accounts
    List {
        /// Call the API once per account to check token validity
        #[arg(long)]
        check: bool,
    },
    /// Check that an account's stored token still works
    Test {
        /// Account name
        name: String,
    },
    /// Remove an account and delete its stored token
    Remove {
        /// Account name
        name: String,
        /// Skip the confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn command_tree_is_valid() {
        Cli::command().debug_assert();
    }
}
