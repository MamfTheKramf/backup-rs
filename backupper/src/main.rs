mod config;

use std::env;

fn main() {
    println!("{:?}", env::current_dir());
    println!("{:?}", config::load_general_config());
}
