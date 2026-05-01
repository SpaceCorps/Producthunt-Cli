# Producthunt.Console

CLI for searching and scraping Product Hunt launches and products via Apify — YAML-first output optimized for LLM agent consumption.

## Install

```bash
dotnet tool install -g Producthunt.Console
```

## Usage

```bash
# Set API key
export APIFY_TOKEN=your-token-here

# Search for products
producthunt search "AI coding"
producthunt search "developer tools" --sort popular --max 20

# Scrape a product page
producthunt scrape "https://www.producthunt.com/posts/some-product"

# Scrape a collection
producthunt scrape "https://www.producthunt.com/topics/developer-tools" --max 30
```
