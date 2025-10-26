use std::fs::Permissions;
use std::io::Error;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
// use interprocess::local_socket::{prelude::*, GenericFilePath, ListenerOptions};
// use std::{io::ErrorKind};
use protos::event::MyEvent;
use protos::event::event_sender_server::EventSenderServer;
use protos::hello::MyGreeter;
use tonic::transport::Server;

use protos::hello::greeter_server::GreeterServer;

use tokio::fs;
use tokio::net::UnixListener;
use tokio::signal;
use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;


async fn create_pipe(path: &str) -> Result<UnixListenerStream, std::io::Error> {
    if fs::metadata(path).await.is_ok() {
        fs::remove_file(path).await?;
    }

    std::fs::create_dir_all(Path::new(path).parent().unwrap())?;

    let uds = UnixListener::bind(path).unwrap();
    fs::set_permissions(path, Permissions::from_mode(0o600)).await?;
    
    Ok(UnixListenerStream::new(uds))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let path = "/tmp/core.socket";

    let stream = create_pipe(path).await?;

    let shutdown_token = CancellationToken::new();
    let task_token = shutdown_token.clone();

    let server_task = tokio::task::spawn(async move {
        let greeter = MyGreeter::default();
        let event = MyEvent::default();

        println!("Server is running at {path}");

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
