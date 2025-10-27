use crate::hello::greeter_client::GreeterClient;
use crate::hello::greeter_server::Greeter;
use hyper_util::rt::TokioIo;
use tonic::transport::{Channel, Error};
use tonic::{Request, Response, Status};
use tower::service_fn;

include_proto!("hello");

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Received a message {:?}", request.into_inner().name);

        let reply = HelloReply {
            message: String::from("ayyy lmao"),
        };

        Ok(Response::new(reply))
    }
}

#[cfg(unix)]
pub async fn create_greeter_client(pipe_name: &String) -> Result<GreeterClient<Channel>, Error> {
    let path = format!("unix:///tmp/{}", pipe_name);
    GreeterClient::connect(path).await
}

#[cfg(windows)]
pub async fn create_greeter_client(pipe_name: &String) -> Result<GreeterClient<Channel>, Error> {
    use std::time::Duration;
    use tokio::{net::windows::named_pipe::ClientOptions, time};
    use tonic::transport::Endpoint;
    use windows::Win32::Foundation::ERROR_PIPE_BUSY;

    let path = format!(r"\\.\pipe\{}", pipe_name.clone());

    let channel = Endpoint::try_from("http://[::]:50051") // Dummy endpoint, only here because we have to add one
        .unwrap()
        .connect_with_connector(service_fn({
            let path = path.clone();
            move |_| {
                let path = path.clone();
                async move {
                    let client = loop {
                        match ClientOptions::new().open(&path) {
                            Ok(client) => break client,
                            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => (),
                            Err(e) => return Err(e),
                        }

                        time::sleep(Duration::from_millis(50)).await;
                    };

                    Ok::<_, std::io::Error>(TokioIo::new(client))
                }
            }
        }))
        .await?;

    Ok(GreeterClient::new(channel))
}
