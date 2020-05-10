// #![feature(custom_test_frameworks)]
// #![test_runner(crate::my_runner)]
// #![feature(async_await)]
// #![feature(unboxed_closures)]
// #![feature(fn_traits)]
use lazy_static::lazy_static;

use futures::channel::oneshot;
use futures::future;
use futures::future::FutureExt;

use tonic::{transport::Server, Request, Response, Status};

use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use grpc::client_server::{
    kvs_command_client::KvsCommandClient,
    kvs_command_request::Cmd,
    kvs_command_response::Status as ServerResponseStatus,
    kvs_command_server::{KvsCommand, KvsCommandServer},
    {Error, Get, KvsCommandRequest, KvsCommandResponse, Ok as ServerOk, Remove, Set},
};

use tests::utils::get_available_port;

lazy_static! {
    static ref PORTS: Mutex<HashMap<u16, bool>> = Mutex::new(HashMap::new());
}

#[derive(Default)]
struct TestKvsServer {}

#[tonic::async_trait]
impl KvsCommand for TestKvsServer {
    async fn send(
        &self,
        request: Request<KvsCommandRequest>,
    ) -> std::result::Result<Response<KvsCommandResponse>, Status> {
        let response: ServerResponseStatus =
            if let KvsCommandRequest { cmd: Some(cmd) } = request.get_ref() {
                match cmd {
                    Cmd::Get { 0: Get { key } } => ServerResponseStatus::Ok {
                        0: ServerOk {
                            msg: format!("get: {}", key),
                        },
                    },
                    Cmd::Set {
                        0: Set { key, value },
                    } => ServerResponseStatus::Ok {
                        0: ServerOk {
                            msg: format!("set: {} {}", key, value),
                        },
                    },
                    Cmd::Remove { 0: Remove { key } } => ServerResponseStatus::Ok {
                        0: ServerOk {
                            msg: format!("remove: {}", key),
                        },
                    },
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

async fn client(
    sender: oneshot::Sender<()>,
    port: u16,
    request: KvsCommandRequest,
    predicate: impl Fn(String) -> (),
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://127.0.0.1:{}", port);
    let channel = tonic::transport::Channel::from_shared(addr)?
        .connect()
        .await?;

    let mut client = KvsCommandClient::new(channel);
    let request = tonic::Request::new(request);

    let response = client.send(request).await.unwrap().into_inner();

    match response {
        KvsCommandResponse {
            status:
                Some(ServerResponseStatus::Ok {
                    0: ServerOk { msg },
                }),
        } => {
            predicate(msg);
        }
        KvsCommandResponse {
            status: Some(ServerResponseStatus::Error { 0: Error { msg } }),
        } => {
            predicate(msg);
        }
        _ => unreachable!("invalid response"),
    }

    sender.send(()).unwrap();

    Ok(())
}

async fn server(rcv: oneshot::Receiver<()>, port: u16) {
    let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();

    let say = TestKvsServer::default();

    let rcv = rcv.map(|_| ());

    Server::builder()
        .add_service(KvsCommandServer::new(say))
        .serve_with_shutdown(
            addr, // tokio::time::delay_for(std::time::Duration::from_secs(5)),
            rcv,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn client_sends_get_message() {
    let (sender, receiver) = oneshot::channel::<()>();

    let request = KvsCommandRequest {
        cmd: Some(Cmd::Get {
            0: Get {
                key: "key1".to_owned(),
            },
        }),
    };
    let expected_response_message = "get: key1".to_owned();

    let predicate = |msg| {
        assert_eq!(msg, expected_response_message);
        assert_ne!(msg, "");
    };

    let mut port = PORTS.lock().unwrap();
    let available_port = get_available_port(&port).unwrap();
    let mut addr = format!("http://127.0.0.1:{}", available_port);

    while tonic::transport::Channel::from_shared(addr).is_err() {
        port.insert(available_port, true);
        let available_port = get_available_port(&port).unwrap();
        addr = format!("http://127.0.0.1:{}", available_port);
    }

    port.insert(available_port, true);
    drop(port);

    let handle = future::join(
        server(receiver, available_port),
        client(sender, available_port, request.clone(), predicate),
    )
    .await;

    // while let (_, Err(Error)) = handle.await {
    //     let (sender, receiver) = oneshot::channel::<()>();
    //     let mut port = PORTS.lock().unwrap();
    //     let available_port = get_available_port(&port).unwrap();
    //     let mut addr = format!("http://127.0.0.1:{}", available_port);

    //     while tonic::transport::Channel::from_shared(addr).is_err() {
    //         port.insert(available_port, true);
    //         let available_port = get_available_port(&port).unwrap();
    //         addr = format!("http://127.0.0.1:{}", available_port);
    //     }

    //     port.insert(available_port, true);
    //     drop(port);

    //     handle = future::join(
    //         server(receiver, available_port),
    //         client(sender, available_port, request.clone(), predicate),
    //     );
    // }

    // Ok(())
}

#[tokio::test]
async fn client_sends_set_message() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = oneshot::channel::<()>();

    let request = KvsCommandRequest {
        cmd: Some(Cmd::Set {
            0: Set {
                key: "key1".to_owned(),
                value: "value1".to_owned(),
            },
        }),
    };
    let expected_response_message = "set: key1 value1".to_owned();
    let predicate = |msg| {
        assert_eq!(msg, expected_response_message);
        assert_ne!(msg, "");
    };

    let mut port = PORTS.lock().unwrap();
    let available_port = get_available_port(&port).unwrap();
    let mut addr = format!("http://127.0.0.1:{}", available_port);

    while tonic::transport::Channel::from_shared(addr).is_err() {
        port.insert(available_port, true);
        let available_port = get_available_port(&port).unwrap();
        addr = format!("http://127.0.0.1:{}", available_port);
    }

    port.insert(available_port, true);
    drop(port);

    future::join(
        server(receiver, available_port),
        client(sender, available_port, request, predicate),
    )
    .await;

    Ok(())
}

#[tokio::test]
async fn client_sends_remove_message() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = oneshot::channel::<()>();

    let request = KvsCommandRequest {
        cmd: Some(Cmd::Remove {
            0: Remove {
                key: "key1".to_owned(),
            },
        }),
    };
    let expected_response_message = "remove: key1".to_owned();

    let predicate = |msg| {
        assert_eq!(msg, expected_response_message);
        assert_ne!(msg, "");
    };

    let mut port = PORTS.lock().unwrap();
    let available_port = get_available_port(&port).unwrap();
    let mut addr = format!("http://127.0.0.1:{}", available_port);

    while tonic::transport::Channel::from_shared(addr).is_err() {
        port.insert(available_port, true);
        let available_port = get_available_port(&port).unwrap();
        addr = format!("http://127.0.0.1:{}", available_port);
    }

    port.insert(available_port, true);
    drop(port);

    future::join(
        server(receiver, available_port),
        client(sender, available_port, request, predicate),
    )
    .await;

    Ok(())
}

#[tokio::test]
async fn client_sends_empty_invalid_message() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let (sender, receiver) = oneshot::channel::<()>();

    let request = KvsCommandRequest { cmd: None };
    let expected_response_message = "error: unknown command".to_owned();

    let predicate = |msg| {
        assert_eq!(msg, expected_response_message);
        assert_ne!(msg, "");
    };

    let mut port = PORTS.lock().unwrap();
    let available_port = get_available_port(&port).unwrap();
    let mut addr = format!("http://127.0.0.1:{}", available_port);

    while tonic::transport::Channel::from_shared(addr).is_err() {
        port.insert(available_port, true);
        let available_port = get_available_port(&port).unwrap();
        addr = format!("http://127.0.0.1:{}", available_port);
    }

    port.insert(available_port, true);
    drop(port);

    future::join(
        server(receiver, available_port),
        client(sender, available_port, request, predicate),
    )
    .await;

    Ok(())
}

// use std::future::Future;

// macro_rules! impl_async_fn {
//     ($(($FnOnce:ident, $FnMut:ident, $Fn:ident, ($($arg:ident: $arg_ty:ident,)*)),)*) => {
//         $(
//             pub trait $FnOnce<$($arg_ty,)*> {
//                 type Output;
//                 type Future: Future<Output = Self::Output> + Send;
//                 fn call_once(self, $($arg: $arg_ty,)*) -> Self::Future;
//             }
//             pub trait $FnMut<$($arg_ty,)*>: $FnOnce<$($arg_ty,)*> {
//                 fn call_mut(&mut self, $($arg: $arg_ty,)*) -> Self::Future;
//             }
//             pub trait $Fn<$($arg_ty,)*>: $FnMut<$($arg_ty,)*> {
//                 fn call(&self, $($arg: $arg_ty,)*) -> Self::Future;
//             }
//             impl<$($arg_ty,)* F, Fut> $FnOnce<$($arg_ty,)*> for F
//             where
//                 F: FnOnce($($arg_ty,)*) -> Fut,
//                 Fut: Future + Send,
//             {
//                 type Output = Fut::Output;
//                 type Future = Fut;
//                 fn call_once(self, $($arg: $arg_ty,)*) -> Self::Future {
//                     self($($arg,)*)
//                 }
//             }
//             impl<$($arg_ty,)* F, Fut> $FnMut<$($arg_ty,)*> for F
//             where
//                 F: FnMut($($arg_ty,)*) -> Fut,
//                 Fut: Future + Send,
//             {
//                 fn call_mut(&mut self, $($arg: $arg_ty,)*) -> Self::Future {
//                     self($($arg,)*)
//                 }
//             }
//             impl<$($arg_ty,)* F, Fut> $Fn<$($arg_ty,)*> for F
//             where
//                 F: Fn($($arg_ty,)*) -> Fut,
//                 Fut: Future + Send,
//             {
//                 fn call(&self, $($arg: $arg_ty,)*) -> Self::Future {
//                     self($($arg,)*)
//                 }
//             }
//         )*
//     }
// }
// impl_async_fn! {
//     // (AsyncFnOnce0, AsyncFnMut0, AsyncFn0, ()),
//     (AsyncFnOnce1, AsyncFnMut1, AsyncFn1, (a0:A0, )),
//     // (AsyncFnOnce2, AsyncFnMut2, AsyncFn2, (a0:A0, a1:A1, )),
//     // (AsyncFnOnce3, AsyncFnMut3, AsyncFn3, (a0:A0, a1:A1, a2:A2, )),
//     // (AsyncFnOnce4, AsyncFnMut4, AsyncFn4, (a0:A0, a1:A1, a2:A2, a3:A3, )),
//     // (AsyncFnOnce5, AsyncFnMut5, AsyncFn5, (a0:A0, a1:A1, a2:A2, a3:A3, a4:A4, )),
//     // (AsyncFnOnce6, AsyncFnMut6, AsyncFn6, (a0:A0, a1:A1, a2:A2, a3:A3, a4:A4, a5:A5, )),
//     // (AsyncFnOnce7, AsyncFnMut7, AsyncFn7, (a0:A0, a1:A1, a2:A2, a3:A3, a4:A4, a5:A5, a6:A6, )),
//     // (AsyncFnOnce8, AsyncFnMut8, AsyncFn8, (a0:A0, a1:A1, a2:A2, a3:A3, a4:A4, a5:A5, a6:A6, a7:A7, )),
//     // (AsyncFnOnce9, AsyncFnMut9, AsyncFn9, (a0:A0, a1:A1, a2:A2, a3:A3, a4:A4, a5:A5, a6:A6, a7:A7, a8:A8, )),
//     // (AsyncFnOnce10, AsyncFnMut10, AsyncFn10, (a0:A0, a1:A1, a2:A2, a3:A3, a4:A4, a5:A5, a6:A6, a7:A7, a8:A8, a9:A9, )),
//     // (AsyncFnOnce11, AsyncFnMut11, AsyncFn11, (a0:A0, a1:A1, a2:A2, a3:A3, a4:A4, a5:A5, a6:A6, a7:A7, a8:A8, a9:A9, a10:A10, )),
//     // (AsyncFnOnce12, AsyncFnMut12, AsyncFn12, (a0:A0, a1:A1, a2:A2, a3:A3, a4:A4, a5:A5, a6:A6, a7:A7, a8:A8, a9:A9, a10:A10, a11:A11, )),
// }

// use std::pin::Pin;

// #[cfg(test)]
// #[tokio::main]
// async fn my_runner(ts: &[&Fn() -> Pin<Box<dyn Future<Output = ()>>>]) {
//     // async fn my_runner<F>(ts: &[F])
//     // where
//     // F: for<'a> AsyncFn1<(), Output = ()>,
//     // {
//     println!("Custom Test Framework running {} tests: ", ts.len());
//     let (sender, receiver) = oneshot::channel::<()>();
//     let available_port = get_available_port().unwrap();

//     // future::join(
//     // server(receiver, available_port);
//     // client(sender, available_port, request, predicate),
//     // )
//     // .await;

//     let f = || async {
//         for t in ts {
//             // let t: &F = t;
//             // AsyncFn1::call(t, ()).await;
//             // F::call(t, ());
//             t().await;
//             // (t as fn(_) -> _).call(()).await;
//         }
//         sender.send(()).unwrap();
//     };
//     future::join(server(receiver, available_port), f()).await;

//     println!("Custom Test Framework: passed");
//     // sender.send(()).unwrap();
//     // };
// }

// pub async fn call<F>(f: F) -> i32
// where
//     F: for<'a> Fn<(&'a i32,)>,
//     for<'a> <F as FnOnce<(&'a i32)>>::Output: Future<Output = i32>,
// {
//     f(&42).await
// }

// #[test_case]
// async fn bar(_: &()) {
//     assert_eq!(1, 2);
// }

// #[test_case]
// fn client_sends_empty_invalid_message() -> Pin<Box<dyn Future<Output = ()>>> {
//     Box::pin(client_sends_empty_invalid_message_async())
// }
// #[tokio::test]
// async fn client_sends_empty_invalid_message() -> Pin<Box<dyn Future<Output = ()>>> {
// async fn client_sends_empty_invalid_message_async() {
//     let (sender, receiver) = oneshot::channel::<()>();

//     let request = KvsCommandRequest { cmd: None };
//     let expected_response_message = "error: unknown command".to_owned();

//     let predicate = |msg| {
//         assert_eq!(msg, expected_response_message);
//         assert_ne!(msg, "");
//     };

//     let available_port = get_available_port().unwrap();

//     future::join(
//         server(receiver, available_port),
//         client(sender, available_port, request, predicate),
//     )
//     .await;
// }

// #[test_case]
// fn client_sends_empty_invalid() -> Pin<Box<dyn Future<Output = ()>>> {
//     Box::pin(client_sends_empty_invalid_async())
// }

// // #[tokio::test]
// async fn client_sends_empty_invalid_async() {
//     let (sender, receiver) = oneshot::channel::<()>();

//     let request = KvsCommandRequest { cmd: None };
//     let expected_response_message = "error: unknown command".to_owned();

//     let predicate = |msg| {
//         assert_eq!(msg, expected_response_message);
//         assert_ne!(msg, "");
//     };

//     let available_port = get_available_port().unwrap();

//     future::join(
//         server(receiver, available_port),
//         client(sender, available_port, request, predicate),
//     )
//     .await;
// }
