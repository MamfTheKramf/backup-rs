use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// Path to the general_config.json file.
    #[arg(short, long, default_value_t = String::from("./general_config.json"))]
    pub general_config: String,

    /// Path to the logger config YAML-file.
    #[arg(long, default_value_t = String::from("./logging_conf.yaml"))]
    pub logger_config: String,

    #[cfg(windows)]
    #[arg(short, long, default_value_t = String::from("./backupper.exe"))]
    /// Path to executable of backupper
    pub backupper: String,

    #[cfg(unix)]
    #[arg(short, long, default_value_t = String::from("./backupper"))]
    /// Path to executable of backupper
    pub backupper: String,

    /// Path to the directory that stores the frontend files
    #[arg(long, default_value_t = String::from("./frontend"))]
    pub frontend: String,

    /// Whether to show rocket_cli_colors or not
    #[arg(short, long)]
    pub rocket_colors: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
