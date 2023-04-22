use chrono::NaiveDateTime;
use clap::{ ArgGroup, Parser, Subcommand };

use crate::config::ProfileSpecifier;

/// Crate for creating and restoring backups.
/// 
/// Allows to only check for specific Backup Profiles if either the name or the uuid are provided, or checks all of them, if nothing is provided.
#[derive(Parser)]
#[command(author, version, about, long_about)]
#[command(group(
    ArgGroup::new("id")
        .required(true)
        .args(["name", "uuid"]),
))]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to general config file.
    #[arg(short, long, default_value_t = String::from("./general_config.json"))]
    pub general_config: String,

    /// Name of Profile to check.
    #[arg(short, long)]
    pub name: Option<String>,

    /// Uuid of Profile to check.
    #[arg(short, long)]
    pub uuid: Option<String>,

    /// Force backup even if it isn't due yet.
    #[arg(short)]
    pub force: bool,

    /// Set to get verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Performs backup
    Backup,

    /// Restore a backup
    Restore(Restore),

    /// Reschedule the backup of the given profile
    Reschedule,

    /// Delete the given profile
    Delete(Delete),
}

#[derive(clap::Args)]
pub struct Restore {
    /// Format: "YYYY-MM-DD HH:MM". Timestamp that has to be preceeded by the backup. If not set, the latest backup is chosen.
    #[arg(short, long, value_parser = valid_time_format)]
    pub timestamp: Option<NaiveDateTime>,
}

#[derive(clap::Args)]
pub struct Delete {
    /// Remove the already created backup files as well
    #[arg(short, long)]
    pub remove_backups: bool,
}

/// Checks that the provided string is in format `YYYY-MM-DD HH:MM`.
/// 
/// # Returns
/// [NaiveDateTime] representing the provided time or an [Err] explaining the issue.
fn valid_time_format(s: &str) -> Result<NaiveDateTime, String> {
    let format = "%Y-%m-%d %H:%M";
    NaiveDateTime::parse_from_str(s, format)
        .or(Err(String::from("Given argument didn't match the format \"YYYY-MM-DD HH:MM\"!")))
}

impl ProfileSpecifier for Args {
    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|name| name.as_str())
    }

    fn uuid(&self) -> Option<&str> {
        self.uuid.as_ref().map(|uuid| uuid.as_str())
    }
}

/// Parses cli-args and returns them.
pub fn get_args() -> Args {
    Args::parse()
}