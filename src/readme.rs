//! The manual an agent reads before driving this CLI.
//!
//! Markdown by default so it can be pasted into a system prompt or a CLAUDE.md;
//! `--json` outputs structured machine-readable metadata.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "producthunt",
            "actor" => ACTOR_NAME,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call",
                "7" => "no_account - run producthunt accounts list or producthunt login",
            },
        });
        return;
    }
    println!("{README}");
}

pub const ACTOR_NAME: &str = "cloud9_ai~producthunt-scraper";

const RULES: &[&str] = &[
    "Authentication requires an Apify token. Configure accounts with 'producthunt login' or pass --account, --api-key, or set APIFY_TOKEN.",
    "Search and scrape calls execute synchronously via the Apify Product Hunt scraper actor and may take 30-60s.",
    "Output is clean YAML by default; pass --json for structured JSON output suitable for jq and automated agents.",
    "On exit code auth_required (3), stop immediately and surface the remediation string to a human. Do not loop.",
    "Non-zero exits produce an error envelope on stderr containing code, message, and remediation hints.",
];

const README: &str = r#"# producthunt - agent operating manual

A native Rust CLI for searching and scraping Product Hunt launches, products, reviews, and collections via the Apify Product Hunt scraper actor (`cloud9_ai/producthunt-scraper`). Results are YAML on stdout, errors are YAML on stderr, and `--json` switches both streams to JSON.

## Authentication & Multi-Account Management

Authentication uses an Apify API token. You can provide credentials in three ways:

1. **Named accounts in the OS keystore** (Recommended):
   Use `producthunt login [<name>]` or `producthunt accounts add <name>` to securely store your Apify token in the macOS Keychain, Windows DPAPI, or Linux Secret Service vault.
   Specify the account when calling commands: `producthunt search "AI" --account work` (or `-a work`).

2. **Environment variable**:
   Set `APIFY_TOKEN=your_token_here`. When no account is explicitly given, this token is automatically used.

3. **Explicit flag**:
   Pass `--api-key <key>` directly to any command.

### Managing accounts

    producthunt login [<name>] [--api-key <key>]  # opens browser to copy API token
    producthunt accounts add <name> --api-key <key> [--force]
    printf %s "$KEY" | producthunt accounts add <name> --api-key-stdin
    producthunt accounts list [--check]
    producthunt accounts test <name>
    producthunt accounts remove <name> --yes

`add` verifies the token against Apify's `/v2/users/me` endpoint before storing it.
`list` displays configured accounts and active keystore backends.
`test` validates a stored token against the live Apify API.
`remove` clears local credentials.

## Searching Product Hunt

Search for products, launches, categories, or keywords:

    producthunt search "developer tools"
    producthunt search "AI coding assistant" --sort popular --max 20
    producthunt search "database" --sort newest --time-frame this-month -a work

### Search options:
- `<QUERY>`: Search terms, keywords, or topics.
- `--max <N>`: Maximum items to return (default: 10).
- `--sort <SORT>`: Sorting criteria: `popular` (default) or `newest`.
- `--time-frame <TF>`: Time window filter (e.g. `today`, `this-week`, `this-month`, `all-time`).
- `-a, --account <ACCOUNT>`: Account name to authenticate with.
- `--api-key <KEY>`: Direct Apify token override.

## Scraping Product Hunt URLs

Extract structured data from a specific Product Hunt post, launch, topic, or collection URL:

    producthunt scrape "https://www.producthunt.com/posts/claude-3-5-sonnet"
    producthunt scrape "https://www.producthunt.com/topics/artificial-intelligence" --max 30

### Scrape options:
- `<URL>`: Full URL of the Product Hunt product page, topic, or collection.
- `--max <N>`: Maximum items to return (default: 10).
- `-a, --account <ACCOUNT>`: Account name to authenticate with.
- `--api-key <KEY>`: Direct Apify token override.

## Identity Context

Inspect current authentication status and user details:

    producthunt me -a work
    producthunt me --api-key $APIFY_TOKEN

## Error Envelopes & Exit Codes

Errors are formatted as structured envelopes on stderr:

    error: The Apify API token was rejected or is invalid.
    code: auth_required
    remediation: producthunt login work --force

Exit codes:

    0  ok
    1  error          unclassified error
    2  network        temporary network failure; retry once
    3  auth_required  authentication failed; do not retry, show remediation
    4  not_found      the requested resource or actor was not found
    5  rate_limited   rate limit exceeded; back off before retrying
    6  invalid_input  invalid parameters or missing required arguments
    7  no_account     no account specified or configured
"#;
