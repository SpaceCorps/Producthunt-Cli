---
title: "Product Hunt CLI"
description: "A blazing fast native command-line tool and agent interface for searching and scraping Product Hunt launches and products via Apify. Built in Rust for developers and autonomous AI agents."
author: "SpaceCorps"
date: "2026-09-24"
canonical: "https://spacecorps.github.io/Producthunt-Cli/index.md"
---

# Product Hunt CLI

A blazing fast native command-line tool and agent interface for searching and scraping Product Hunt launches, products, reviews, and collections via the Apify Product Hunt scraper actor (`cloud9_ai/producthunt-scraper`). Built in Rust for developers and autonomous AI agents.

## Quickstart

```bash
# Set Apify API token
export APIFY_TOKEN=your-token-here

# Or log in to the OS keystore
producthunt login work

# Search for products
producthunt search "AI coding"
producthunt search "developer tools" --sort popular --max 20

# Scrape a product page
producthunt scrape "https://www.producthunt.com/posts/some-product"

# Scrape a collection
producthunt scrape "https://www.producthunt.com/topics/developer-tools" --max 30
```

## Features

- **Blazing Fast Native Rust**: Sub-millisecond startup times with zero runtime dependencies.
- **AI Agent Native**: Structured JSON output (`--json`) and standardized error envelopes.
- **Secure Keystore Integration**: Token storage in native macOS Keychain, Windows DPAPI, and Linux Secret Service.
- **Multi-Account Workspaces**: Isolate personal and company Apify accounts safely with `--account <name>`.

## When to Use This CLI

Use the `producthunt` CLI whenever you need to:
- Discover trending startups and new software releases on Product Hunt.
- Extract structured product metadata (taglines, vote counts, launch links, makers).
- Automate market research and competitive intelligence pipelines using LLM agents.
- Scrape topic catalogs and curated product collections.

## Documentation Links

- [llms.txt](https://spacecorps.github.io/Producthunt-Cli/llms.txt)
- [Full Agent Manual](https://spacecorps.github.io/Producthunt-Cli/llms-full.txt)
- [Pricing](https://spacecorps.github.io/Producthunt-Cli/pricing.md)
- [Authentication Guide](https://spacecorps.github.io/Producthunt-Cli/auth.md)
- [GitHub Repository](https://github.com/SpaceCorps/Producthunt-Cli)
