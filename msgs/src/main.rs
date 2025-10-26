use std::io;
use tonic::{Request, Response};

use protos::event::{EventReply, EventReq, event_sender_client::EventSenderClient};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let path = "unix:///tmp/core.socket";

    let mut client = EventSenderClient::connect(path).await.unwrap();

    loop {
        let mut msg = String::new();
        io::stdin().read_line(&mut msg)?;

        if msg == "break".to_string() {
            break;
        }

        let msg = msg.trim_ascii_end().to_string();

        let request = Request::new(EventReq {
            msg//: "EventReqMessage".to_string(),
        });

        let response = client.send_event(request).await;

        println!(
            "Server answered: {:?}",
            response
                .unwrap_or_else(|_| Response::new(EventReply {
                    reply: "err".to_string()
                }))
                .into_inner()
                .reply
        );
    }

    Ok(())
}
