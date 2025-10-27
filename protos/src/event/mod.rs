use tonic::{Request, Response, Status};
use crate::event::event_sender_server::EventSender;

include_proto!("event");

#[derive(Default)]
pub struct MyEvent {}

#[tonic::async_trait]
impl EventSender for MyEvent {
    async fn send_event(
        &self,
        request: Request<EventReq>,
    ) -> Result<Response<EventReply>, Status> {
        println!("Received a message {:?}", request.into_inner().msg);

        let reply = EventReply {
            reply: String::from("ayyy lmao"),
        };

        Ok(Response::new(reply))
    }
}
