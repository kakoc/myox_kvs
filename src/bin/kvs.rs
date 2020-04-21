// #![cfg_attr(test, allow(unused_imports, warnings))]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, warnings))]

use clap::{load_yaml, App};

use std::process::exit;

fn main() {
    let yaml = load_yaml!("../../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("get", matches) => {
            // matches.unwrap().value_of("key");
            // println!("{:?}", matches);

            eprintln!("unimplemented");
            exit(1);
        }
        ("set", matches) => {
            eprintln!("unimplemented");
            exit(1);
        }
        ("rm", matches) => {
            eprintln!("unimplemented");
            exit(1);
        }

        _ => panic!(),
    }
}
