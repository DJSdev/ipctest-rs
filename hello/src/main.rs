use protos::hello::{HelloReply, HelloRequest, create_greeter_client};
use std::io;
use tonic::{Request, Response};


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let pipe_name = "core-socket".to_string();

    let mut client = create_greeter_client(&pipe_name).await.unwrap();

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
