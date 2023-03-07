mod cli_args;
mod config;

use std::env;

fn main() {
    let args = cli_args::get_args();
    println!("{:?}", env::current_dir());
    println!("{:?}", config::load_general_config());
}
