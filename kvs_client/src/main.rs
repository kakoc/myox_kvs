// #![cfg_attr(test, allow(unused_imports, warnings))]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, warnings))]

use clap::{load_yaml, App};
use log::{debug, info, LevelFilter};

use std::env::current_dir;
use std::net::SocketAddr;
use std::{collections::HashMap, process::exit};

// use grpc::client_server::kvs_command_client::KvsCommandClient;
// use grpc::client_server::kvs_command_request::Cmd;
// use grpc::client_server::kvs_command_response::Msg;
// use grpc::client_server::{Error, Get, KvsCommandRequest, Ok, Remove, Set};
use grpc::client_server::{
    kvs_command_client::KvsCommandClient,
    kvs_command_request::Cmd,
    kvs_command_response::Status as ServerResponseStatus,
    kvs_command_server::{KvsCommand, KvsCommandServer},
    {Error, Get, KvsCommandRequest, KvsCommandResponse, Ok as ServerOk, Remove, Set},
};

use kvs::KvStore;

const DEFAULT_ADDR: &'static str = "http://127.0.0.1:4000";

// fn build_command(matches: clap::ArgMatches) -> KvsCommandRequest {
//     match matches {
//         ("get", Some(matches)) => {
//             let key = matches.value_of("key").expect("get: key expected");

//             KvsCommandRequest {
//                 cmd: "get".to_string(),
//                 key,
//                 value: "",
//             }
//         }
//     }
// }
async fn create_grpc_client(
    addr: String,
) -> std::result::Result<
    KvsCommandClient<tonic::transport::channel::Channel>,
    Box<dyn std::error::Error>,
> {
    let channel = tonic::transport::Channel::from_shared(addr)?
        .connect()
        .await?;

    Ok(KvsCommandClient::new(channel))
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let yaml = load_yaml!("../client_cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // let addr = matches
    //     .value_of("addr")
    //     .unwrap_or("127.0.0.1:4000")
    //     .parse::<SocketAddr>()?;

    // tonic
    // info!("try to connect to a server with addr: {}", addr);
    // let addr = format!("http://{}", addr);
    // let channel = tonic::transport::Channel::from_shared(addr)?
    //     .connect()
    //     .await?;

    // let mut client = KvsCommandClient::new(channel);

    match matches.subcommand() {
        ("get", Some(matches)) => {
            match (
                &matches.value_of("key"),
                &matches.value_of("addr"),
                &matches.args.len(),
            ) {
                (Some(key), Some(addr), 2) => {
                    let addr = format!("http://{}", addr);
                    info!("try to connect to a server with addr: {}", addr);
                    let mut client = create_grpc_client(addr).await?;

                    let request = tonic::Request::new(KvsCommandRequest {
                        cmd: Some(Cmd::Get {
                            0: Get {
                                key: key.to_string(),
                            },
                        }),
                    });
                    let response = client.send(request).await?.into_inner();

                    debug!("Response: {:?}", response);

                    match response {
                        KvsCommandResponse {
                            status:
                                Some(ServerResponseStatus::Ok {
                                    0: ServerOk { msg },
                                }),
                        } => {
                            println!("{}", msg);
                            Ok(())
                        }
                        KvsCommandResponse {
                            status: Some(ServerResponseStatus::Error { 0: Error { msg } }),
                        } => {
                            println!("Key not found");

                            Ok(())
                        }
                        _ => unreachable!("invalid response"),
                    }
                }
                (Some(key), None, 1) => {
                    info!("try to connect to a server with addr: {}", DEFAULT_ADDR);
                    let mut client = create_grpc_client(DEFAULT_ADDR.to_string()).await?;

                    let request = tonic::Request::new(KvsCommandRequest {
                        cmd: Some(Cmd::Get {
                            0: Get {
                                key: key.to_string(),
                            },
                        }),
                    });
                    let response = client.send(request).await?.into_inner();

                    debug!("Response: {:?}", response);

                    match response {
                        KvsCommandResponse {
                            status:
                                Some(ServerResponseStatus::Ok {
                                    0: ServerOk { msg },
                                }),
                        } => {
                            println!("{}", msg);
                            Ok(())
                        }
                        KvsCommandResponse {
                            status: Some(ServerResponseStatus::Error { 0: Error { msg } }),
                        } => {
                            println!("Key not found");

                            Ok(())
                        }
                        _ => unreachable!("invalid response"),
                    }
                }
                _ => exit(1),
            }
        }
        ("set", Some(matches)) => {
            match (
                &matches.value_of("key"),
                &matches.value_of("value"),
                &matches.value_of("addr"),
                &matches.args.len(),
            ) {
                (Some(key), Some(value), Some(addr), 3) => {
                    let addr = format!("http://{}", addr);
                    info!("try to connect to a server with addr: {}", addr);
                    let mut client = create_grpc_client(addr).await?;

                    let request = tonic::Request::new(KvsCommandRequest {
                        cmd: Some(Cmd::Set {
                            0: Set {
                                key: key.to_string(),
                                value: value.to_string(),
                            },
                        }),
                    });
                    let response = client.send(request).await?.into_inner();

                    debug!("Response: {:?}", response);

                    match response {
                        KvsCommandResponse {
                            status:
                                Some(ServerResponseStatus::Ok {
                                    0: ServerOk { msg },
                                }),
                        } => Ok(()),
                        KvsCommandResponse {
                            status: Some(ServerResponseStatus::Error { 0: Error { msg } }),
                        } => {
                            println!("Key was not insterted");

                            exit(1);
                        }
                        _ => unreachable!("invalid response"),
                    }
                }
                (Some(key), Some(value), None, 2) => {
                    info!("try to connect to a server with addr: {}", DEFAULT_ADDR);
                    let mut client = create_grpc_client(DEFAULT_ADDR.to_string()).await?;

                    let request = tonic::Request::new(KvsCommandRequest {
                        cmd: Some(Cmd::Set {
                            0: Set {
                                key: key.to_string(),
                                value: value.to_string(),
                            },
                        }),
                    });
                    let response = client.send(request).await?.into_inner();

                    debug!("Response: {:?}", response);

                    match response {
                        KvsCommandResponse {
                            status:
                                Some(ServerResponseStatus::Ok {
                                    0: ServerOk { msg },
                                }),
                        } => Ok(()),
                        KvsCommandResponse {
                            status: Some(ServerResponseStatus::Error { 0: Error { msg } }),
                        } => {
                            println!("Key was not insterted");

                            exit(1);
                        }
                        _ => unreachable!("invalid response"),
                    }
                }
                _ => exit(1),
            }
        }
        ("rm", Some(matches)) => {
            match (
                &matches.value_of("key"),
                &matches.value_of("addr"),
                &matches.args.len(),
            ) {
                (Some(key), Some(addr), 2) => {
                    let addr = format!("http://{}", addr);
                    info!("try to connect to a server with addr: {}", addr);
                    let mut client = create_grpc_client(addr).await?;

                    let request = tonic::Request::new(KvsCommandRequest {
                        cmd: Some(Cmd::Remove {
                            0: Remove {
                                key: key.to_string(),
                            },
                        }),
                    });
                    let response = client.send(request).await?.into_inner();

                    debug!("Response: {:?}", response);

                    match response {
                        KvsCommandResponse {
                            status:
                                Some(ServerResponseStatus::Ok {
                                    0: ServerOk { msg },
                                }),
                        } => Ok(()),
                        KvsCommandResponse {
                            status: Some(ServerResponseStatus::Error { 0: Error { msg } }),
                        } => {
                            eprintln!("Key not found");

                            exit(1);
                        }
                        _ => unreachable!("invalid response"),
                    }
                }
                (Some(key), None, 1) => {
                    info!("try to connect to a server with addr: {}", DEFAULT_ADDR);
                    let mut client = create_grpc_client(DEFAULT_ADDR.to_string()).await?;

                    let request = tonic::Request::new(KvsCommandRequest {
                        cmd: Some(Cmd::Remove {
                            0: Remove {
                                key: key.to_string(),
                            },
                        }),
                    });
                    let response = client.send(request).await?.into_inner();

                    debug!("Response: {:?}", response);

                    match response {
                        KvsCommandResponse {
                            status:
                                Some(ServerResponseStatus::Ok {
                                    0: ServerOk { msg },
                                }),
                        } => Ok(()),
                        KvsCommandResponse {
                            status: Some(ServerResponseStatus::Error { 0: Error { msg } }),
                        } => {
                            eprintln!("Key not found");

                            exit(1);
                        }
                        _ => unreachable!("invalid response"),
                    }
                }
                _ => exit(1),
            }
        }
        _ => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client() {
        assert_eq!(1, 1);
    }
}
