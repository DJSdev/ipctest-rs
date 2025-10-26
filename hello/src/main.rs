use std::io;
use tonic::{Request, Response};

use protos::hello::{greeter_client::GreeterClient, HelloReply, HelloRequest};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let path = "unix:///tmp/core.socket";

    let mut client = GreeterClient::connect(path).await.unwrap();

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
