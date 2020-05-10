// #![cfg_attr(test, allow(unused_imports, warnings))]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, warnings))]

use clap::{load_yaml, App};
use log::{error, info, LevelFilter};
use tonic::{transport::Server, Request, Response, Status};
// mod grpc::client_server;

use std::env::current_dir;
use std::net::SocketAddr;
use std::{
    process::exit,
    sync::{Arc, Mutex},
};

use grpc::client_server::kvs_command_request::Cmd;
use grpc::client_server::kvs_command_response::Status as ServerResponseStatus;
use grpc::client_server::kvs_command_server::{KvsCommand, KvsCommandServer};
use grpc::client_server::{
    Error, Get, KvsCommandRequest, KvsCommandResponse, Ok as ServerOk, Remove, Set,
};
use kvs::{KvStore, KvsEngine, Result, SledKvsEngine};

pub struct MySay {
    store: Mutex<Box<dyn KvsEngine + Send>>,
}

#[tonic::async_trait]
impl KvsCommand for MySay {
    // our rpc impelemented as function
    async fn send(
        &self,
        request: Request<KvsCommandRequest>,
    ) -> std::result::Result<Response<KvsCommandResponse>, Status> {
        let response: ServerResponseStatus =
            if let KvsCommandRequest { cmd: Some(cmd) } = request.get_ref() {
                match cmd {
                    Cmd::Get { 0: Get { key } } => {
                        if let Ok(Some(string)) = self
                            .store
                            .lock()
                            .expect("mutex not poisoned")
                            .get(key.to_owned())
                        // .expect("error during fetching for get request")
                        {
                            ServerResponseStatus::Ok {
                                0: ServerOk { msg: string },
                            }
                        } else {
                            ServerResponseStatus::Error {
                                0: Error {
                                    msg: "get: key not found".to_string(),
                                },
                            }
                        }

                        // ServerResponseStatus::Ok {
                        //     0: ServerOk {
                        //         msg: format!("get: {}", key),
                        //     },
                        // }
                    }
                    Cmd::Set {
                        0: Set { key, value },
                    } => {
                        if let Ok(()) = self
                            .store
                            .lock()
                            .expect("mutex not poisoned")
                            .set(key.to_owned(), value.to_owned())
                        {
                            ServerResponseStatus::Ok {
                                0: ServerOk { msg: "".to_owned() },
                            }
                        } else {
                            ServerResponseStatus::Error {
                                0: Error {
                                    msg: "set: error during set".to_string(),
                                },
                            }
                        }
                        // ServerResponseStatus::Ok {
                        //     0: ServerOk {
                        //         msg: format!("set: {} {}", key, value),
                        //     },
                        // }
                    }
                    Cmd::Remove { 0: Remove { key } } => {
                        if let Ok(()) = self
                            .store
                            .lock()
                            .expect("mutex not poisoned")
                            .remove(key.to_owned())
                        {
                            ServerResponseStatus::Ok {
                                0: ServerOk { msg: "".to_owned() },
                            }
                        } else {
                            ServerResponseStatus::Error {
                                0: Error {
                                    msg: "remove: error during remove".to_string(),
                                },
                            }
                        }
                        // ServerResponseStatus::Ok {
                        //     0: ServerOk {
                        //         msg: format!("remove: {}", key),
                        //     },
                        // }
                    }
                    _ => unreachable!(),
                }
            } else {
                ServerResponseStatus::Error {
                    0: Error {
                        msg: "error: unknown command".to_string(),
                    },
                }
            };

        Ok(Response::new(KvsCommandResponse {
            status: Some(response),
        }))
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let yaml = load_yaml!("../server_cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let addr = matches
        .value_of("addr")
        .unwrap_or("127.0.0.1:4000")
        .parse::<SocketAddr>()?;

    let engine = match matches.value_of("engine") {
        Some("sled") => "sled",
        _ => "kvs",
    };

    match try_find_config(&current_dir()?) {
        Ok(config) if config.contains(engine) => {}
        Ok(not_valid) if !not_valid.contains(engine) => {
            error!(
                "previous storage: {}\nnow current passed storage: {}",
                not_valid, engine
            );
            exit(1);
        }
        Err(_) => {
            save_engine_config(&current_dir()?, engine);
        }
        _ => unreachable!(),
    }

    let mut store: Mutex<Box<dyn KvsEngine + Send>> = if engine == "sled" {
        Mutex::new(Box::new(SledKvsEngine::new(&current_dir()?)))
    } else {
        Mutex::new(Box::new(KvStore::open(&current_dir()?).unwrap()))
    };

    // let mut store: Mutex<Box<dyn KvsEngine + Send>> =
    // Mutex::new(Box::new(KvStore::open(&current_dir()?).unwrap()));

    info!("version: {}", env!("CARGO_PKG_VERSION"));
    info!("addr: {}", addr);
    info!("engine: {}", engine);

    let say = MySay { store };
    info!("Server listening on {}", addr);
    // adding our service to our server.
    Server::builder()
        .add_service(KvsCommandServer::new(say))
        .serve(addr)
        .await?;

    Ok(())
}

fn try_find_config(path: &std::path::PathBuf) -> Result<String> {
    Ok(std::fs::read_to_string(path.join("kvs.conf"))?)
}

fn save_engine_config(path: &std::path::PathBuf, engine: &str) -> Result<()> {
    Ok(std::fs::write(path.join("kvs.conf"), engine)?)
}
