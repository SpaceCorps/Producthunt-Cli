//! `producthunt scrape <URL>` command.

use std::io::IsTerminal;

use serde_json::Value;

use crate::account;
use crate::cli::ScrapeArgs;
use crate::commands::print;
use crate::error::Result;

pub fn run(args: ScrapeArgs) -> Result<()> {
    let resolved = account::resolve(args.account.as_deref(), args.api_key.as_deref())?;

    if std::io::stderr().is_terminal() {
        eprintln!("Scraping Product Hunt (this may take 30–60s)...");
    }

    let mut input = serde_json::Map::new();
    input.insert("dateUrl".into(), Value::String(args.url));
    input.insert("maxResults".into(), Value::Number(args.max.into()));

    let results = resolved.client().scrape(&Value::Object(input))?;
    print(results);
    Ok(())
}
