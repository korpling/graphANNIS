extern crate lalrpop;

use lalrpop::Configuration;

fn main() {
    Configuration::new().use_cargo_dir_conventions().process().unwrap();
}
