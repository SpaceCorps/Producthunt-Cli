//! Command dispatcher and common printing helper.

pub mod accounts;
pub mod login;
pub mod scrape;
pub mod search;

use std::io::IsTerminal;

use serde_json::Value;

use crate::account;
use crate::cli::{AuthArgs, Command};
use crate::error::Result;
use crate::output;
use crate::readme;

pub fn run(cmd: Command) -> Result<()> {
    match cmd {
        Command::Search(args) => search::run(args),
        Command::Scrape(args) => scrape::run(args),
        Command::Me(args) => me(args),
        Command::Login(args) => login::run(args),
        Command::Accounts(cmd) => accounts::run(cmd),
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
    }
}

pub fn print(v: Value) {
    output::write(&v);
}

fn me(args: AuthArgs) -> Result<()> {
    let resolved = account::resolve(args.account.as_deref(), args.api_key.as_deref())?;
    if std::io::stderr().is_terminal() {
        eprintln!("Authenticating with Apify as account '{}'...", resolved.name);
    }
    let me = resolved.client().me()?;
    print(me);
    Ok(())
}
