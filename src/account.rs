//! Resolves the target TV device: either via explicit `--account <name>`,
//! explicit `--host <ip>` (and `--token <token>`), or environment variables.

use crate::client::SamsungTvClient;
use crate::config::{self, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct ResolvedTv {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub host: String,
    pub token: Option<String>,
}

impl ResolvedTv {
    pub fn client(&self, verbose: bool) -> Result<SamsungTvClient> {
        SamsungTvClient::connect(&self.host, self.token.as_deref(), verbose)
    }
}

pub fn resolve(account: Option<&str>, host: Option<&str>, token: Option<&str>) -> Result<ResolvedTv> {
    // 1. Explicit --host overrides all accounts
    if let Some(host) = host.map(str::trim).filter(|s| !s.is_empty()) {
        let tok = token
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| std::env::var("SAMSUNG_TV_TOKEN").ok().filter(|s| !s.trim().is_empty()));
        return Ok(ResolvedTv { name: None, host: host.to_string(), token: tok });
    }

    // 2. Explicit --account
    if let Some(requested) = account.map(str::trim).filter(|s| !s.is_empty()) {
        let config = config::load()?;
        let Some((name, account_cfg)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No TV account named '{requested}'."))
                .detail(describe_accounts(&config))
                .fix("samsung-artmode accounts list"));
        };

        let key = if let Some(tok) = token.map(str::trim).filter(|s| !s.is_empty()) {
            Some(tok.to_string())
        } else {
            secrets::store()?.get(&secrets::account_key(name))?
        };

        return Ok(ResolvedTv {
            name: Some(name.clone()),
            host: account_cfg.host.clone(),
            token: key.filter(|k| !k.trim().is_empty()),
        });
    }

    // 3. Fallback to SAMSUNG_TV_HOST environment variable
    if let Some(env_host) =
        std::env::var("SAMSUNG_TV_HOST").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    {
        let tok = token
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| std::env::var("SAMSUNG_TV_TOKEN").ok().filter(|s| !s.trim().is_empty()));
        return Ok(ResolvedTv { name: None, host: env_host, token: tok });
    }

    // 4. No TV target provided: report configured accounts and exit with NoAccount
    let config = config::load()?;
    Err(Error::new(ErrorCode::NoAccount, "No TV specified. Pass --account <name> or --host <ip>.")
        .detail(describe_accounts(&config))
        .fix("samsung-artmode accounts list"))
}

pub fn describe_accounts(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No TV accounts are configured yet. Run 'samsung-artmode pair' or 'samsung-artmode accounts add <name> --host <ip>'.".into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| {
            if v.model.trim().is_empty() {
                format!("{k} ({})", v.host)
            } else {
                format!("{k} ({} - {})", v.host, v.model)
            }
        })
        .collect();
    format!("Configured TV accounts: {}", listed.join(", "))
}
