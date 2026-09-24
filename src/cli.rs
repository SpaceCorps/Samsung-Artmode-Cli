//! Clap command definitions and argument parsing hierarchy.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "samsung-artmode",
    version,
    about = "CLI for Samsung The Frame TV Art Mode - pair, upload, list, mattes, slideshows and brightness",
    after_help = "An LLM agent should start with: samsung-artmode agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    /// TV account name (see 'samsung-artmode accounts list')
    #[arg(short = 'a', long, global = true, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// TV IP address or hostname (overrides account host or SAMSUNG_TV_HOST)
    #[arg(long, global = true, value_name = "HOST")]
    pub host: Option<String>,

    /// TV authentication token (overrides account token or SAMSUNG_TV_TOKEN)
    #[arg(long, global = true, value_name = "TOKEN")]
    pub token: Option<String>,

    /// Print WebSocket messages and network trace to stderr
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Pair with a Samsung Frame TV and acquire an auth token
    Pair {
        /// TV IP address or hostname (auto-discovers on local network if omitted)
        host: Option<String>,
        /// Optional name to save TV in config and store token in keystore
        #[arg(long)]
        name: Option<String>,
        /// Scan/connection timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },
    /// Get current Art Mode status (on/off)
    Status,
    /// List artwork images on the TV
    List {
        /// Filter by category (MY-C0002=My Photos, MY-C0004=Favorites, MY-C0008=Store)
        #[arg(long)]
        category: Option<String>,
    },
    /// Get the currently displayed artwork
    Current,
    /// Select an image to display on the TV
    Select {
        /// Content ID of the image (e.g. SAM-F0206)
        content_id: String,
        /// Immediately display the image even if TV is not currently in art mode
        #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
        show: bool,
    },
    /// Upload an image (PNG or JPEG) to the TV
    Upload {
        /// Path to the image file (PNG or JPEG)
        file: String,
        /// Matte to apply (e.g. none, flexible_polar, shadowbox_black)
        #[arg(long)]
        matte: Option<String>,
        /// Immediately select the uploaded image as active artwork
        #[arg(long)]
        select: bool,
    },
    /// Delete an image from the TV
    Delete {
        /// Content ID of the image to delete
        content_id: String,
    },
    /// Add or remove an image from favorites
    Favorite {
        /// Content ID of the image
        content_id: String,
        /// Remove from favorites instead of adding
        #[arg(long)]
        remove: bool,
    },
    /// Display brightness controls
    #[command(subcommand)]
    Brightness(Brightness),
    /// Matte/frame border controls
    #[command(subcommand)]
    Matte(Matte),
    /// Configure slideshow settings
    Slideshow {
        /// Disable slideshow
        #[arg(long)]
        off: bool,
        /// Slideshow interval in minutes (e.g. 1, 5, 10, 30, 60)
        #[arg(long)]
        interval: Option<u32>,
        /// Category to slideshow (default: MY-C0002)
        #[arg(long, default_value = "MY-C0002")]
        category: String,
        /// Shuffle the slideshow order
        #[arg(long)]
        shuffle: bool,
    },
    /// Manage configured TV devices and auth tokens
    #[command(subcommand)]
    Accounts(Accounts),
    /// Print the operating manual for an LLM agent driving this CLI
    AgentReadme,
}

#[derive(Subcommand)]
pub enum Brightness {
    /// Get current brightness (0-10)
    Get,
    /// Set brightness (0-10)
    Set {
        /// Brightness value between 0 and 10
        value: u8,
    },
}

#[derive(Subcommand)]
pub enum Matte {
    /// List available matte styles
    List,
    /// Apply a matte to an image
    Set {
        /// Content ID of the image
        content_id: String,
        /// Matte ID to apply (e.g. flexible_polar, shadowbox_black, none)
        matte_id: String,
    },
}

#[derive(Subcommand)]
pub enum Accounts {
    /// List configured TV devices
    List,
    /// Add or update a TV device
    Add {
        /// Name of the TV device (e.g. living-room, bedroom)
        name: String,
        /// IP address or hostname of the TV
        #[arg(long)]
        host: String,
        /// Authentication token (if already paired)
        #[arg(long)]
        token: Option<String>,
        /// MAC address (optional)
        #[arg(long)]
        mac: Option<String>,
        /// TV Model (optional)
        #[arg(long)]
        model: Option<String>,
        /// Replace existing account if it already exists
        #[arg(long)]
        force: bool,
    },
    /// Test connection to a configured TV
    Test {
        /// Name of the TV device
        name: String,
    },
    /// Remove a configured TV device and delete its token from keystore
    Remove {
        /// Name of the TV device
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_tree_is_valid() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
