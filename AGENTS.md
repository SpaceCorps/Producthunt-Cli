# AGENTS.md

Notes for developers and autonomous agents extending or maintaining `producthunt`.

`producthunt` is a native Rust CLI for searching and scraping Product Hunt launches, products, reviews, and collections via the Apify scraper actor (`cloud9_ai/producthunt-scraper`). It replaces a .NET global tool prototype (`Producthunt.Console` by Niels Bosma) while preserving and improving its interface: search and scrape capabilities, YAML-first output with `--json` support, OS keystore credential security, multi-account isolation, and deterministic error envelopes.

For the operational manual the *agent* reads at runtime, run `producthunt agent-readme` (or pass `--json`). That text lives in `src/readme.rs` and is the tool's self-documenting interface. This document is for humans and agents editing the codebase.

## Developer Commands

```bash
cargo build --release              # target/release/producthunt
cargo test --locked                # unit tests + in-process mock integration tests
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install binary on PATH
```

### Isolated Testing Environment

Always use an isolated temporary config directory and the plaintext secret store when testing so you never touch real OS keystores:

```bash
export PRODUCTHUNT_CONFIG_DIR=$(mktemp -d) PRODUCTHUNT_SECRET_STORE=plaintext
```

| Variable | Effect |
| --- | --- |
| `PRODUCTHUNT_CONFIG_DIR` | Overrides the config and secrets storage directory |
| `PRODUCTHUNT_SECRET_STORE` | Forces a keystore backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `PRODUCTHUNT_ALLOW_PLAINTEXT_STORE=1` | Explicit opt-in permitting the unencrypted fallback when no OS keystore exists |
| `PRODUCTHUNT_API_URL` / `APIFY_API_URL` | Overrides the Apify API base URL (used by `tests/cli.rs` for local mock server) |
| `APIFY_TOKEN` | Direct token environment variable fallback when no `--account` or `--api-key` is supplied |

## Codebase Layout

```
src/
  main.rs          # Argument parsing, --json pre-scan, clap error formatting to envelopes
  cli.rs           # Clap derive hierarchy, arguments, subcommands, and help strings
  client.rs        # Blocking HTTP client (ureq + rustls), actor execution, status -> ErrorCode
  error.rs         # ErrorCode enum and structured Error envelope { code, error, detail, remediation }
  output.rs        # YAML default (serde_norway), JSON (--json), write_error, obj! macro
  account.rs       # Multi-account resolution, identity probe against /v2/users/me
  config.rs        # config.yaml layout, atomic file writes, 0600 file permissions, cross-process lock
  secrets.rs       # macOS Keychain, Linux secret-tool, Windows DPAPI, plaintext fallback
  readme.rs        # Embedded agent-readme documentation and rules
  commands/
    mod.rs         # Dispatcher and print helper
    search.rs      # producthunt search <QUERY> [--sort] [--max] [--time-frame]
    scrape.rs      # producthunt scrape <URL> [--max]
    login.rs       # producthunt login [NAME]
    accounts.rs    # producthunt accounts add|list|test|remove
tests/
  cli.rs           # In-process TCP mock HTTP server verifying offline CLI execution
docs/              # Complete GitHub Pages website, SEO metadata, and agent discovery suite
```

## Architectural Tenets

1. **Native Speed & Zero Runtime Dependencies:**
   - Standalone binary compiles to ~5-8 MB with `lto = "fat"`, `panic = "abort"`, and `strip = true`.
   - Sub-millisecond cold starts (1–3 ms) ensure agent tool-calling loops experience zero runtime initialization delays.

2. **Blocking HTTP (No Async Runtime Bloat):**
   - The CLI performs at most 1–2 sequential requests per command invocation.
   - Built on `ureq` (3.4) with `rustls` and connection pooling; avoids the startup overhead and dependency tree of `tokio`.
   - 300-second global timeout accommodates synchronous execution of Apify scraper actor runs.

3. **Deterministic Multi-Account Safety:**
   - Commands accept `--account <name>` (`-a <name>`), `--api-key <key>`, or `APIFY_TOKEN`.
   - OS keystore isolation prevents cross-organization pollution or credential leakage.

4. **Agentic Output Protocol:**
   - Standard output is pure YAML by default, switched to clean JSON via `--json`.
   - Errors emit structured JSON/YAML envelopes to `stderr` with stable exit codes matching the `code` field.
   - Interactive hints (e.g. "Searching Product Hunt (this may take 30–60s)...") only render when `stderr.is_terminal()`, keeping piped pipelines 100% clean.
