use async_stream::stream;
use interprocess::local_socket::tokio::Stream;
use std::{io, pin::Pin};
// use std::fs::Permissions;
// use std::os::unix::fs::PermissionsExt;
// use std::path::Path;
// use interprocess::local_socket::{prelude::*, GenericFilePath, ListenerOptions};
// use std::{io::ErrorKind};
use protos::event::MyEvent;
use protos::event::event_sender_server::EventSenderServer;
use protos::hello::MyGreeter;
use tonic::{
    IntoRequest,
    transport::{Server, server::Connected},
};

use protos::hello::greeter_server::GreeterServer;

// use tokio::fs;
// use tokio::net::UnixListener;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::windows::named_pipe::ServerOptions,
    signal,
};
// use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;

use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, ToNsName, tokio::Listener,
    traits::tokio::Listener as TraitListener,
};

mod named_pipe;

// async fn create_pipe(path: &str) -> Result<UnixListenerStream, io::Error> {
//     if fs::metadata(path).await.is_ok() {
//         fs::remove_file(path).await?;
//     }

//     std::fs::create_dir_all(Path::new(path).parent().unwrap())?;

//     let uds = UnixListener::bind(path).unwrap();
//     fs::set_permissions(path, Permissions::from_mode(0o600)).await?;

//     Ok(UnixListenerStream::new(uds))
// }

struct Pipe {
    inner: Stream,
}

impl Pipe {
    fn new(stream: Stream) -> Self {
        Self { inner: stream }
    }
}

impl Connected for Pipe {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {
        ()
    }
}

impl AsyncRead for Pipe {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for Pipe {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
}

fn create_pipe(name: &str) -> Result<Stream, io::Error> {
    let name = format!(r"\\.\pipe\{}", name);

    stream! {
        let mut server = ServerOptions::new().first_pipe_instance(true).create(path);

        loop {
            server.connect().await;

            let client = Pipe::new(server);

            yield Ok(client);

            server = ServerOptions::new().first_pipe_instance(true).create(path);
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let path = "/tmp/core.socket";

    let stream = create_pipe(path)?;

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
