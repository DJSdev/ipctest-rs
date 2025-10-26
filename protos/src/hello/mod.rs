use tonic::{Request, Response, Status};
use crate::hello::greeter_server::{Greeter};

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
