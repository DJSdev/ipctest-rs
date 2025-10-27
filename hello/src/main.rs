use std::{io, time::Duration};
use hyper_util::rt::TokioIo;
use tokio::{net::windows::named_pipe::ClientOptions, time};
#[cfg(unix)]
use tokio::net::UnixStream;
use tonic::{transport::{Endpoint, Uri}, Request, Response};

use protos::hello::{greeter_client::GreeterClient, HelloReply, HelloRequest};

use tower::service_fn;

use windows::Win32::Foundation::ERROR_PIPE_BUSY;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    #[cfg(unix)]
    let path = "unix:///tmp/core.socket";
    #[cfg(windows)]
    let path = r"\\.\pipe\core-socket";

    let channel = Endpoint::try_from("http://[::]:50051")
        .unwrap()
        .connect_with_connector(service_fn(move |_| async move {
            let client = loop {
                match ClientOptions::new().open(path) {
                    Ok(client) => break client,
                    Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => (),
                    Err(e) => return Err(e),
                }

                time::sleep(Duration::from_millis(50)).await;
            };

            Ok::<_, std::io::Error>(TokioIo::new(client))
        }))
        .await.unwrap();

    #[cfg(unix)]
    let mut client = GreeterClient::connect(path).await.unwrap();

    #[cfg(windows)]
    let mut client = GreeterClient::new(channel);

    loop {
        let mut name = String::new();
        io::stdin().read_line(&mut name)?;

        if name == "break".to_string() {
            break;
        }

        let name = name.trim_ascii_end().to_string();

        let request = Request::new(HelloRequest {
            name//: "GreeterMsg".to_string(),
        });

        let response = client.say_hello(request).await;

        println!(
            "Server answered: {:?}",
            response
                .unwrap_or_else(|_| Response::new(HelloReply {
                    message: "err".to_string()
                }))
                .into_inner()
                .message
        );
    }

    Ok(())
}
