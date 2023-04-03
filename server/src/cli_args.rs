use clap::Parser;


#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// Path to the general_config.json file.
    #[arg(short, long, default_value_t = String::from("./general_config.json"))]
    pub general_config: String,
}

pub fn parse_args() -> Args {
    Args::parse()
}
