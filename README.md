# Product Hunt CLI (`producthunt`)

[![CI](https://github.com/SpaceCorps/Producthunt-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Producthunt-Cli/actions/workflows/ci.yml)
[![Pages](https://github.com/SpaceCorps/Producthunt-Cli/actions/workflows/pages.yml/badge.svg)](https://spacecorps.github.io/Producthunt-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust: 2024](https://img.shields.io/badge/Rust-2024%20(1.89+)-orange.svg)](https://www.rust-lang.org)

A blazing fast, native command-line tool and autonomous agent interface for searching and scraping Product Hunt launches, products, reviews, and collections via Apify. Built in Rust 2024 with zero runtime dependencies, sub-millisecond cold starts, native OS keystore integration, and YAML/JSON dual output.

> Re-architected from the original [.NET prototype](https://github.com/nielsbosma/Producthunt.Console) by Niels Bosma into a high-performance native binary under the SpaceCorps engineering collective.

---

## Highlights

- **Native Speed & Zero Dependencies:** Single standalone static binary (~5–8 MB). Starts in 1–3 ms with no .NET, Node.js, or Python runtime needed.
- **Agentic Protocol:** Pure YAML output by default for readable terminal inspection; supply `--json` for structured, ordered JSON streams suitable for `jq` and LLM tool loops.
- **Native OS Keystores:** Stores Apify API tokens in macOS Keychain, Windows DPAPI, or Linux Secret Service (`secret-tool`). Plaintext files are strictly opt-in.
- **Deterministic Multi-Account Workspaces:** Easily manage and isolate multiple personal or organization accounts with `--account <name>`.
- **Self-Documenting Agent Interface:** Run `producthunt agent-readme [--json]` to output complete operating instructions and error codes directly to LLM agents with zero network roundtrips.

---

## Installation

### From Source (via Cargo)

```bash
cargo install --git https://github.com/SpaceCorps/Producthunt-Cli --locked
```

### Precompiled Binaries

Download standalone binaries from the [Releases](https://github.com/SpaceCorps/Producthunt-Cli/releases) page:
- **macOS:** Apple Silicon (`aarch64-apple-darwin`) & Intel (`x86_64-apple-darwin`)
- **Linux:** Static Musl binaries for `x86_64` and `aarch64`
- **Windows:** `x86_64-pc-windows-msvc` (`.exe`)

---

## Quickstart & Authentication

Authenticate with your Apify API token (from [console.apify.com/account/integrations](https://console.apify.com/account/integrations)):

### 1. Interactive Login (Stores in OS Keystore)

```bash
# Opens browser to Apify tokens page and prompts securely
producthunt login work
```

### 2. Non-Interactive / CI/CD Setup

```bash
# Read token from stdin
printf %s "$APIFY_TOKEN" | producthunt accounts add work --api-key-stdin

# Or pass directly via environment variable
export APIFY_TOKEN="apify_api_..."
```

---

## Usage Examples

### Search Product Hunt

Search for products, launches, categories, or keywords:

```bash
# Search trending launches
producthunt search "AI coding"

# Search with custom sort and limit
producthunt search "developer tools" --sort popular --max 20

# Filter by time frame and specify account
producthunt search "database" --sort newest --time-frame this-month --account work

# Output structured JSON for piping into jq
producthunt search "robotics" --json | jq '.[].name'
```

### Scrape Product Posts & Collections

Extract structured data from any Product Hunt URL:

```bash
# Scrape a specific product page
producthunt scrape "https://www.producthunt.com/posts/claude-3-5-sonnet"

# Scrape an entire topic collection
producthunt scrape "https://www.producthunt.com/topics/artificial-intelligence" --max 30

# Output raw JSON
producthunt scrape "https://www.producthunt.com/posts/cursor" --json
```

### Manage Accounts

```bash
# List configured accounts and active keystore backend
producthunt accounts list

# Verify stored tokens against the live Apify API
producthunt accounts list --check

# Test account connectivity and inspect identity
producthunt accounts test work

# Remove an account from local storage
producthunt accounts remove work --yes
```

### Inspect Identity Context

```bash
# View active Apify user profile
producthunt me -a work
```

---

## Agentic Interface & Error Envelopes

Every error prints a structured envelope to `stderr` and exits with a stable, machine-readable status code:

```json
{
  "error": "The Apify API token was rejected or is invalid.",
  "code": "auth_required",
  "detail": "HTTP 401: {\"error\":{\"message\":\"Token is invalid!\",\"type\":\"INVALID_TOKEN\"}}",
  "remediation": "producthunt login work --force or set APIFY_TOKEN=<token>"
}
```

### Exit Codes

| Code | Name | Agent Action |
| :--- | :--- | :--- |
| `0` | `ok` | Success. Parse `stdout`. |
| `1` | `error` | Unclassified failure. Stop and report. |
| `2` | `network` | Transport timeout or connection error. Retry once. |
| `3` | `auth_required` | Token invalid, revoked, or missing. Surface remediation string to human. |
| `4` | `not_found` | Resource or actor not found. Do not retry. |
| `5` | `rate_limited` | Apify rate limits reached. Back off before retrying. |
| `6` | `invalid_input` | Missing required parameters or invalid syntax. |
| `7` | `no_account` | No account specified. Run `producthunt accounts list`. |

---

## Documentation & Discovery

- **Website & Documentation:** [https://spacecorps.github.io/Producthunt-Cli/](https://spacecorps.github.io/Producthunt-Cli/)
- **Agent Guide (`llms.txt`):** [https://spacecorps.github.io/Producthunt-Cli/llms.txt](https://spacecorps.github.io/Producthunt-Cli/llms.txt)
- **Comprehensive Agent Manual:** [https://spacecorps.github.io/Producthunt-Cli/llms-full.txt](https://spacecorps.github.io/Producthunt-Cli/llms-full.txt)
- **Agent Card Discovery:** [https://spacecorps.github.io/Producthunt-Cli/.well-known/agent-card.json](https://spacecorps.github.io/Producthunt-Cli/.well-known/agent-card.json)
- **Authentication Guide:** [https://spacecorps.github.io/Producthunt-Cli/auth.md](https://spacecorps.github.io/Producthunt-Cli/auth.md)
- **Pricing & Licensing:** [https://spacecorps.github.io/Producthunt-Cli/pricing.md](https://spacecorps.github.io/Producthunt-Cli/pricing.md)

---

## License

MIT License &copy; 2026 SpaceCorps. Developed by the SpaceCorps open-source engineering collective.
Original prototype created by [Niels Bosma](https://github.com/nielsbosma).
