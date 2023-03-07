use clap::{ ArgGroup, Parser };

/// Crate for creating backups.
/// Allows to only check for specific Backup Profiles if either the name or the uuid are provided, or checks all of them, if nothing is provided.
#[derive(Parser)]
#[command(author, version, about, long_about)]
#[command(group(
    ArgGroup::new("id")
        .required(true)
        .args(["name", "uuid"]),
))]
pub struct Args {
    /// Name of Profile to check.
    #[arg(short, long)]
    pub name: Option<String>,

    /// Uuid of Profile to check.
    #[arg(short, long)]
    pub uuid: Option<String>,
}

/// Parses cli-args and returns them.
pub fn get_args() -> Args {
    Args::parse()
}