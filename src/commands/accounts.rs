//! Account management commands for configured TV devices.

use serde_json::json;

use crate::client::SamsungTvClient;
use crate::config::{self, AccountConfig};
use crate::error::{Error, ErrorCode, Result};
use crate::output;
use crate::secrets;

pub fn list() -> Result<()> {
    let config = config::load()?;
    let accounts: Vec<_> = config
        .sorted()
        .into_iter()
        .map(|(name, acct)| {
            json!({
                "name": name,
                "host": acct.host,
                "mac": if acct.mac.is_empty() { None } else { Some(&acct.mac) },
                "model": if acct.model.is_empty() { None } else { Some(&acct.model) },
                "addedAt": acct.added_at,
            })
        })
        .collect();

    output::write(&json!({ "accounts": accounts }));
    Ok(())
}

pub fn add(
    name: &str,
    host: &str,
    token: Option<&str>,
    mac: Option<&str>,
    model: Option<&str>,
    force: bool,
) -> Result<()> {
    let _lock = config::lock()?;
    let mut cfg = config::load()?;

    if !force && cfg.find(name).is_some() {
        return Err(Error::invalid(format!("Account '{name}' already exists. Pass --force to replace it.")));
    }

    if let Some(tok) = token.filter(|t| !t.trim().is_empty()) {
        secrets::store()?.set(&secrets::account_key(name), tok.trim())?;
    }

    let acct = AccountConfig {
        host: host.trim().to_string(),
        mac: mac.unwrap_or("").trim().to_string(),
        model: model.unwrap_or("").trim().to_string(),
        added_at: config::now_utc(),
    };

    cfg.accounts.insert(name.to_string(), acct);
    config::save(&cfg)?;

    output::write(&json!({
        "status": "ok",
        "account": name,
        "host": host,
        "tokenConfigured": token.is_some(),
    }));
    Ok(())
}

pub fn test(name: &str) -> Result<()> {
    let cfg = config::load()?;
    let Some((actual_name, acct)) = cfg.find(name) else {
        return Err(Error::new(ErrorCode::NotFound, format!("No TV account named '{name}'."))
            .fix("samsung-artmode accounts list"));
    };

    let token = secrets::store()?.get(&secrets::account_key(actual_name))?;
    let mut client = SamsungTvClient::connect(&acct.host, token.as_deref(), false)?;
    let status = client.get_artmode_status()?;

    output::write(&json!({
        "status": "ok",
        "account": actual_name,
        "host": acct.host,
        "connected": true,
        "tvStatus": status,
    }));
    Ok(())
}

pub fn remove(name: &str, yes: bool) -> Result<()> {
    if !yes {
        // Human confirmation in interactive terminal
        eprintln!("Are you sure you want to remove TV account '{name}'? Pass --yes to confirm.");
    }

    let _lock = config::lock()?;
    let mut cfg = config::load()?;

    let (stored_name, _) = match cfg.find(name) {
        Some((k, v)) => (k.clone(), v.clone()),
        None => {
            return Err(Error::new(ErrorCode::NotFound, format!("No TV account named '{name}'."))
                .fix("samsung-artmode accounts list"));
        }
    };

    let _ = secrets::store()?.delete(&secrets::account_key(&stored_name));
    cfg.accounts.shift_remove(&stored_name);
    config::save(&cfg)?;

    output::write(&json!({
        "status": "ok",
        "removed": stored_name,
    }));
    Ok(())
}
