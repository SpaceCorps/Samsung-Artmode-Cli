//! Samsung The Frame TV Art Mode command implementations.

use std::path::Path;
use std::time::Duration;

use serde_json::json;

use crate::account;
use crate::client::SamsungTvClient;
use crate::discovery;
use crate::error::{Error, Result};
use crate::output;

pub fn pair(host_arg: Option<&str>, name: Option<&str>, timeout_secs: u64, verbose: bool) -> Result<()> {
    let host = match host_arg {
        Some(h) if !h.trim().is_empty() => h.trim().to_string(),
        _ => {
            eprintln!("Scanning local network for Samsung TVs ({}s timeout)...", timeout_secs);
            let tvs = discovery::discover_tvs(Duration::from_secs(timeout_secs))?;

            if tvs.is_empty() {
                return Err(Error::network("No Samsung TVs found on the local network.")
                    .detail("Ensure the TV is powered on and connected to the same local network.")
                    .fix("Specify the IP directly: samsung-artmode pair <ip>"));
            }

            if tvs.len() == 1 {
                let tv = &tvs[0];
                eprintln!("Found Samsung TV at {} {}", tv.host, tv.name.as_deref().unwrap_or(""));
                tv.host.clone()
            } else {
                let list_str: Vec<_> =
                    tvs.iter().map(|t| format!("{} ({})", t.host, t.name.as_deref().unwrap_or("Samsung TV"))).collect();
                return Err(Error::invalid("Multiple Samsung TVs found on the network.")
                    .detail(format!("Available TVs: {}", list_str.join(", ")))
                    .fix(format!("samsung-artmode pair {}", tvs[0].host)));
            }
        }
    };

    eprintln!("Connecting to TV at {host}... Check TV screen and press 'Allow' if prompted.");
    let token = SamsungTvClient::pair(&host, verbose, Duration::from_secs(timeout_secs.max(20)))?;

    if let Some(account_name) = name.filter(|n| !n.trim().is_empty()) {
        let _lock = crate::config::lock()?;
        let mut cfg = crate::config::load()?;
        crate::secrets::store()?.set(&crate::secrets::account_key(account_name), &token)?;
        let acct = crate::config::AccountConfig {
            host: host.clone(),
            mac: String::new(),
            model: String::new(),
            added_at: crate::config::now_utc(),
        };
        cfg.accounts.insert(account_name.to_string(), acct);
        crate::config::save(&cfg)?;

        output::write(&json!({
            "status": "ok",
            "host": host,
            "token": token,
            "account": account_name,
            "paired": true,
        }));
    } else {
        output::write(&json!({
            "status": "ok",
            "host": host,
            "token": token,
            "paired": true,
            "remediation": format!("samsung-artmode accounts add living-room --host {host} --token {token}"),
        }));
    }

    Ok(())
}

pub fn status(account: Option<&str>, host: Option<&str>, token: Option<&str>, verbose: bool) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.get_artmode_status()?;
    output::write(&res);
    Ok(())
}

pub fn list(
    account: Option<&str>,
    host: Option<&str>,
    token: Option<&str>,
    category: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.get_content_list(category)?;
    output::write(&res);
    Ok(())
}

pub fn current(account: Option<&str>, host: Option<&str>, token: Option<&str>, verbose: bool) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.get_current_artwork()?;
    output::write(&res);
    Ok(())
}

pub fn select(
    account: Option<&str>,
    host: Option<&str>,
    token: Option<&str>,
    content_id: &str,
    show: bool,
    verbose: bool,
) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.select_image(content_id, show)?;
    output::write(&res);
    Ok(())
}

pub fn upload(
    account: Option<&str>,
    host: Option<&str>,
    token: Option<&str>,
    file: &str,
    matte: Option<&str>,
    select: bool,
    verbose: bool,
) -> Result<()> {
    let file_path = Path::new(file);
    if !file_path.exists() {
        return Err(Error::invalid(format!("File not found: {file}")));
    }

    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let content_id = client.upload_image(file_path, matte)?;

    if select {
        let _ = client.select_image(&content_id, true);
    }

    output::write(&json!({
        "status": "ok",
        "contentId": content_id,
        "selected": select,
    }));
    Ok(())
}

pub fn delete(
    account: Option<&str>,
    host: Option<&str>,
    token: Option<&str>,
    content_id: &str,
    verbose: bool,
) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let _ = client.delete_image_list(content_id)?;
    output::write(&json!({
        "status": "ok",
        "deleted": content_id,
    }));
    Ok(())
}

pub fn favorite(
    account: Option<&str>,
    host: Option<&str>,
    token: Option<&str>,
    content_id: &str,
    remove: bool,
    verbose: bool,
) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.change_favorite(content_id, remove)?;
    output::write(&res);
    Ok(())
}

pub fn brightness_get(account: Option<&str>, host: Option<&str>, token: Option<&str>, verbose: bool) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.get_brightness()?;
    output::write(&res);
    Ok(())
}

pub fn brightness_set(
    account: Option<&str>,
    host: Option<&str>,
    token: Option<&str>,
    value: u8,
    verbose: bool,
) -> Result<()> {
    if value > 10 {
        return Err(Error::invalid("Brightness value must be between 0 and 10."));
    }
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.set_brightness(value)?;
    output::write(&res);
    Ok(())
}

pub fn matte_list(account: Option<&str>, host: Option<&str>, token: Option<&str>, verbose: bool) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.get_matte_list()?;
    output::write(&res);
    Ok(())
}

pub fn matte_set(
    account: Option<&str>,
    host: Option<&str>,
    token: Option<&str>,
    content_id: &str,
    matte_id: &str,
    verbose: bool,
) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.change_matte(content_id, matte_id)?;
    output::write(&res);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn slideshow(
    account: Option<&str>,
    host: Option<&str>,
    token: Option<&str>,
    off: bool,
    interval: Option<u32>,
    category: &str,
    shuffle: bool,
    verbose: bool,
) -> Result<()> {
    let target = account::resolve(account, host, token)?;
    let mut client = target.client(verbose)?;
    let res = client.set_slideshow(off, interval, category, shuffle)?;
    output::write(&res);
    Ok(())
}
