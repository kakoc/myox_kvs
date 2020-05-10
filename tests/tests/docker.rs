use futures::future::{self, Future};
// use std::future::Future;
// use tokio::prelude::Future;
// use tokio::runtime::current_thread;

// use futures::stream::Stream;

// use dockertest::error::DockerError;
// use dockertest::waitfor::RunningWait;
// use dockertest::{Composition, DockerTest, Image, PullPolicy, Source};
// use std::rc::Rc;

#[test]
fn docker() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // std::thread::spawn(|| {
    //     std::process::Command::new("docker")
    //         .args(&[
    //             "run",
    //             "-d",
    //             "--network",
    //             "host",
    //             "-p",
    //             "4001:4001",
    //             "kvs-server",
    //         ])
    //         .stdout(std::process::Stdio::null())
    //         .spawn();
    // });
    // .output()
    // .expect("failed to execute process");

    // std::thread::sleep(std::time::Duration::from_secs(333));

    // let body = reqwest::get("http://127.0.0.1:4001")
    //     .unwrap()
    //     .text()
    //     .unwrap();

    // println!("{:?}", body);

    // let source = Source::Local;
    // let mut test = DockerTest::new().with_default_source(source);

    // let repo = "kvs-server".to_string();
    // let img = Image::with_repository(&repo);
    // let hello_world = Composition::with_image(img);

    // test.add_composition(hello_world);

    // test.run(|_ops| {
    //     let h = _ops.handle("kvs-server").unwrap();
    //     std::thread::sleep(std::time::Duration::from_secs(3333));
    //     // "http://127.0.0.1:4001"
    //     println!("=> {} {}", h.ip(), h.host_port(111));
    //     let body = reqwest::get(format!("http://{}:4001", h.ip()).as_str())
    //         .unwrap()
    //         .text()
    //         .unwrap();

    //     println!("{:?}", body);
    //     println!("--->>>>> {:?}", h.ip());
    //     assert!(1 == 2);

    //     // std::thread::sleep(std::time::Duration::from_secs(1222));

    //     assert!(true);
    // });

    // assert!(1 == 7);
    // let source = Source::Local;

    // let image = Image::with_repository("kvs-server");
    // let client = Rc::new(shiplift::Docker::new());

    // let mut rt = Runtime::new()?;

    // let mut rt = current_thread::Runtime::new().expect("failed to start tokio runtime");

    // let f: Future<Item = bool, Error = DockerError> = client
    //     .images()
    //     .get(&format!("{}:{}", "kvs-server", "latest"))
    //     .inspect()
    //     .then(|res| future::ok(res.is_ok()));

    // let res = rt
    //     .block_on(
    //         client
    //             .images()
    //             .get(&format!("{}:{}", "kvs-server", "latest"))
    //             .inspect()
    //             // .then::<dyn Future<Item = bool, Error = DockerError>>(|res| {
    //             //     future::ok(res.is_ok())
    //             // }),
    //     )
    //     .expect("failed to set image id");

    // client
    //     .images()
    //     .get(&format!("{}:{}", "kvs-server", ""))
    //     .inspect()
    //     .await;
    // .await;
    // .then(|res| future::ok(res.is_ok()));
    // image.does_image_exist().await;

    Ok(())
}
