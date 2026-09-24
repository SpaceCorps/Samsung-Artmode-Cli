//! Command dispatcher for samsung-artmode.

pub mod accounts;
pub mod art;

use crate::cli::{Brightness, Command, Matte};
use crate::error::Result;
use crate::readme;

pub fn run(
    command: Command,
    account: Option<String>,
    host: Option<String>,
    token: Option<String>,
    verbose: bool,
) -> Result<()> {
    match command {
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
        Command::Login { host: pair_host, name, timeout }
        | Command::Pair { host: pair_host, name, timeout } => {
            art::pair(pair_host.as_deref().or(host.as_deref()), name.as_deref(), timeout, verbose)
        }

        Command::Status => art::status(account.as_deref(), host.as_deref(), token.as_deref(), verbose),
        Command::List { category } => {
            art::list(account.as_deref(), host.as_deref(), token.as_deref(), category.as_deref(), verbose)
        }
        Command::Current => art::current(account.as_deref(), host.as_deref(), token.as_deref(), verbose),
        Command::Select { content_id, show } => {
            art::select(account.as_deref(), host.as_deref(), token.as_deref(), &content_id, show, verbose)
        }
        Command::Upload { file, matte, select } => {
            art::upload(account.as_deref(), host.as_deref(), token.as_deref(), &file, matte.as_deref(), select, verbose)
        }
        Command::Delete { content_id } => {
            art::delete(account.as_deref(), host.as_deref(), token.as_deref(), &content_id, verbose)
        }
        Command::Favorite { content_id, remove } => {
            art::favorite(account.as_deref(), host.as_deref(), token.as_deref(), &content_id, remove, verbose)
        }
        Command::Brightness(b) => match b {
            Brightness::Get => art::brightness_get(account.as_deref(), host.as_deref(), token.as_deref(), verbose),
            Brightness::Set { value } => {
                art::brightness_set(account.as_deref(), host.as_deref(), token.as_deref(), value, verbose)
            }
        },
        Command::Matte(m) => match m {
            Matte::List => art::matte_list(account.as_deref(), host.as_deref(), token.as_deref(), verbose),
            Matte::Set { content_id, matte_id } => {
                art::matte_set(account.as_deref(), host.as_deref(), token.as_deref(), &content_id, &matte_id, verbose)
            }
        },
        Command::Slideshow { off, interval, category, shuffle } => art::slideshow(
            account.as_deref(),
            host.as_deref(),
            token.as_deref(),
            off,
            interval,
            &category,
            shuffle,
            verbose,
        ),
        Command::Accounts(accts) => match accts {
            crate::cli::Accounts::List => accounts::list(),
            crate::cli::Accounts::Add { name, host: acct_host, token: acct_tok, mac, model, force } => {
                accounts::add(&name, &acct_host, acct_tok.as_deref(), mac.as_deref(), model.as_deref(), force)
            }
            crate::cli::Accounts::Test { name } => accounts::test(&name),
            crate::cli::Accounts::Remove { name, yes } => accounts::remove(&name, yes),
        },
    }
}
