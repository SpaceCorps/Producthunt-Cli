---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the Product Hunt CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for Product Hunt CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the Product Hunt CLI.

## Overview
The Product Hunt CLI interfaces with Apify's REST API (v2) to run the `cloud9_ai/producthunt-scraper` actor. Authentication is token-based, using personal API tokens issued through your Apify account console. Tokens can be stored in the host operating system's native keychain or supplied directly via environment variables and standard input.

## Prerequisites
- An Apify account ([apify.com](https://apify.com))
- A valid Apify API token generated from the integrations page (`https://console.apify.com/account/integrations`)
- Product Hunt CLI installed (`cargo install --git https://github.com/SpaceCorps/Producthunt-Cli --locked`)

## Authentication Methods

### 1. Interactive Browser Login (`producthunt login`)
The recommended flow for local developer machines:
```bash
producthunt login [account_name]
```
1. The CLI launches your system browser to `https://console.apify.com/account/integrations`.
2. You copy or generate your personal API token.
3. Paste the token into the CLI prompt (input characters are masked).
4. The CLI validates the token with a live request to `GET /v2/users/me`.
5. Upon confirmation, the token is securely saved to the native OS keyring under the account name (defaults to `default`).

### 2. Non-Interactive / Headless Login
For headless CI/CD environments, Docker containers, or autonomous agent runners:
```bash
echo "$APIFY_TOKEN" | producthunt login [account_name] --api-key-stdin
```
Or pass the token directly as a CLI flag:
```bash
producthunt login [account_name] --api-key "$APIFY_TOKEN"
```

### 3. Environment Variable Fallback
The CLI automatically checks for credentials in the environment when no account flag is specified:
```bash
export APIFY_TOKEN="apify_api_..."
producthunt search "AI coding"
```

### 4. Direct Command Flag
Pass `--api-key` directly to any command:
```bash
producthunt search "developer tools" --api-key "apify_api_..."
```

## Multi-Account Management
Inspect and verify accounts using:
```bash
producthunt accounts list --check
producthunt accounts test [account_name]
producthunt accounts remove [account_name] --yes
```

## Error Handling
When authentication fails, commands exit with non-zero exit codes and output standardized error payloads:
- `auth_required` (exit code 3): Invalid, missing, or revoked token.
- `no_account` (exit code 7): Specified account does not exist in keystore, or no credentials provided.
- `rate_limited` (exit code 5): Apify API rate limits reached.

## Security Best Practices
1. **Never Commit Tokens**: Keep `.env` or plaintext token files out of version control.
2. **Use OS Keystore**: The CLI automatically utilizes macOS Keychain, Windows DPAPI, or Linux Secret Service.
3. **Machine Verification**: When writing agent automation scripts, always pass `--json` to reliably capture error codes.
