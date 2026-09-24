//! Account resolution and identity probing.
//!
//! Keys can be provided via:
//! 1. `--account <name>` (`-a <name>`) referencing a stored OS keystore credential.
//! 2. `--api-key <key>` passed directly on the command line.
//! 3. `APIFY_TOKEN` environment variable.

use serde_json::Value;

use crate::client::Client;
use crate::config::{self, AccountConfig, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct Resolved {
    pub name: String,
    #[allow(dead_code)]
    pub config: Option<AccountConfig>,
    pub api_key: String,
}

impl Resolved {
    pub fn client(&self) -> Client {
        Client::new(&self.api_key)
    }
}

pub fn resolve(requested_account: Option<&str>, explicit_key: Option<&str>) -> Result<Resolved> {
    if let Some(key) = explicit_key.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(Resolved { name: "(flag)".to_string(), config: None, api_key: key.to_string() });
    }

    let config = config::load()?;

    if let Some(requested) = requested_account.map(str::trim).filter(|s| !s.is_empty()) {
        let Some((name, account)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'."))
                .detail(describe(&config))
                .fix("producthunt accounts list"));
        };

        let key = secrets::store()?.get(&secrets::account_key(name))?;

        let Some(api_key) = key.filter(|k| !k.trim().is_empty()) else {
            return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{name}' has no stored API key."))
                .detail("The config entry exists but the keystore has nothing under it.")
                .fix(format!("producthunt accounts add {name} --api-key <key>")));
        };

        return Ok(Resolved { name: name.clone(), config: Some(account.clone()), api_key });
    }

    if let Some(env_token) = std::env::var("APIFY_TOKEN").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        return Ok(Resolved { name: "(env)".to_string(), config: None, api_key: env_token });
    }

    if config.accounts.is_empty() {
        return Err(Error::new(
            ErrorCode::NoAccount,
            "No account or API key specified. Pass --account <name>, --api-key <key>, or set APIFY_TOKEN.",
        )
        .detail("No accounts are configured yet. Run 'producthunt login <name>' or 'producthunt accounts add <name>'.")
        .fix("producthunt login"));
    }

    Err(Error::new(
        ErrorCode::NoAccount,
        "No account specified. Pass --account <name>, --api-key <key>, or set APIFY_TOKEN.",
    )
    .detail(describe(&config))
    .fix("producthunt accounts list"))
}

fn describe(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No accounts are configured yet. Run 'producthunt accounts add <name> --api-key <key>'.".into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| if v.identity.trim().is_empty() { k.clone() } else { format!("{k} ({})", v.identity) })
        .collect();
    format!("Configured accounts: {}", listed.join(", "))
}

/// Reads identity from Apify `/v2/users/me`.
pub mod identity {
    use super::Value;

    const IDENTITY_KEYS: &[&str] = &["username", "user_name", "email", "name", "id"];

    pub fn describe(me: &Value) -> String {
        if let Some(data) = me.get("data").filter(|d| d.is_object())
            && let Some(ident) = first_string(data, IDENTITY_KEYS)
        {
            return ident;
        }
        first_string(me, IDENTITY_KEYS).unwrap_or_default()
    }

    pub fn user_id(me: &Value) -> String {
        if let Some(data) = me.get("data").filter(|d| d.is_object())
            && let Some(id) = first_string(data, &["id", "userId", "user_id"])
        {
            return id;
        }
        first_string(me, &["id", "userId", "user_id"]).unwrap_or_default()
    }

    fn first_string(v: &Value, names: &[&str]) -> Option<String> {
        names
            .iter()
            .filter_map(|n| v.get(*n).and_then(Value::as_str))
            .find(|s| !s.trim().is_empty())
            .map(str::to_string)
    }

    #[cfg(test)]
    mod tests {
        use serde_json::json;

        #[test]
        fn reads_apify_user_me() {
            let me = json!({
                "data": {
                    "id": "u_12345",
                    "username": "johndoe",
                    "email": "john@example.com"
                }
            });
            assert_eq!(super::describe(&me), "johndoe");
            assert_eq!(super::user_id(&me), "u_12345");
        }

        #[test]
        fn reads_flat_apify_user() {
            let me = json!({
                "id": "u_999",
                "username": "spaceuser"
            });
            assert_eq!(super::describe(&me), "spaceuser");
            assert_eq!(super::user_id(&me), "u_999");
        }
    }
}
