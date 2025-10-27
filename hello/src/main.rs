use hyper_util::rt::TokioIo;
use protos::hello::{
    HelloReply, HelloRequest, create_greeter_client, greeter_client::GreeterClient,
};
use std::{io, time::Duration};
use tonic::{Request, Response};

#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(windows)]
use tokio::net::windows::named_pipe::ClientOptions;
#[cfg(windows)]
use windows::Win32::Foundation::ERROR_PIPE_BUSY;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let pipe_name = "core-socket".to_string();

    let mut client = create_greeter_client(pipe_name).await.unwrap();

    loop {
        let mut name = String::new();
        io::stdin().read_line(&mut name)?;

        if name == "break".to_string() {
            break;
        }

        let name = name.trim_ascii_end().to_string();

        let request = Request::new(HelloRequest {
            name, //: "GreeterMsg".to_string(),
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
