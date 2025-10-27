use protos::event::MyEvent;
use protos::event::event_sender_server::EventSenderServer;
use protos::hello::MyGreeter;
use tonic::{
    transport::Server,
};

use protos::hello::greeter_server::GreeterServer;

use tokio::signal;

use tokio_util::sync::CancellationToken;

#[cfg(unix)]
use shared::pipe::server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pipe_name = "core-socket";

    let stream = server::create_pipe(pipe_name).await?;

    let shutdown_token = CancellationToken::new();
    let task_token = shutdown_token.clone();

    let server_task = tokio::task::spawn(async move {
        let greeter = MyGreeter::default();
        let event = MyEvent::default();

        println!("Server is running at {pipe_name}");

        Server::builder()
            .add_service(GreeterServer::new(greeter))
            .add_service(EventSenderServer::new(event))
            .serve_with_incoming_shutdown(stream, task_token.cancelled())
            .await
    });

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("Shutdown received");
                shutdown_token.cancel();
                break;
            }
        }
    }

    println!("Stopping server");
    let _ = server_task.await?;

    println!("Exiting cleanly");
    Ok(())
}
