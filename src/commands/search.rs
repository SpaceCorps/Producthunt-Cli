//! `producthunt search <QUERY>` command.

use std::io::IsTerminal;

use serde_json::Value;

use crate::account;
use crate::cli::SearchArgs;
use crate::commands::print;
use crate::error::Result;

pub fn run(args: SearchArgs) -> Result<()> {
    let resolved = account::resolve(args.account.as_deref(), args.api_key.as_deref())?;

    if std::io::stderr().is_terminal() {
        eprintln!("Searching Product Hunt (this may take 30–60s)...");
    }

    let mut input = serde_json::Map::new();
    input.insert("searchQuery".into(), Value::String(args.query));
    input.insert("maxResults".into(), Value::Number(args.max.into()));
    input.insert("sortBy".into(), Value::String(args.sort));
    if let Some(tf) = args.time_frame.filter(|s| !s.trim().is_empty()) {
        input.insert("timeFrame".into(), Value::String(tf));
    }

    let results = resolved.client().search(&Value::Object(input))?;
    print(results);
    Ok(())
}
