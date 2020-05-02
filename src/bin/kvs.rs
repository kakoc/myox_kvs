// #![cfg_attr(test, allow(unused_imports, warnings))]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, warnings))]

use clap::{load_yaml, App};

use std::env::current_dir;
use std::process::exit;

use myox_kvs::{KVSResult, KvStore};

fn main() -> KVSResult<()> {
    let yaml = load_yaml!("../../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("get", Some(matches)) => {
            let key = matches.value_of("key").expect("get: key expected");

            // let s = std::fs::read_to_string(&current_dir()?.join("1.log"))?;
            // println!("{}", s);

            let mut store = KvStore::open(&current_dir()?)?;
            // println!("get---------");
            // println!("{:?}", &current_dir());
            // let r = store.get(key.to_owned())?;
            // println!("@@@@@@@@@@@@@@@@@@@@@@@@");
            // println!("{:?}", r);

            if let Some(value) = store.get(key.to_owned())? {
                println!("{}", value);

                Ok(())
            } else {
                println!("Key not found");

                Ok(())
            }
        }
        ("set", Some(matches)) => {
            let key = matches.value_of("key").expect("set: key is expected");
            let value = matches.value_of("value").expect("set: value is expected");

            let mut store = KvStore::open(&current_dir()?)?;

            store.set(key.to_owned(), value.to_owned())?;

            Ok(())
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("key").expect("rm: key is expected");

            let mut store = KvStore::open(&current_dir()?)?;
            if let Ok(()) = store.remove(key.to_owned()) {
                // Ok(())
                exit(0);
            } else {
                println!("Key not found");
                exit(1);
            }
        }

        _ => panic!(),
    }
}
