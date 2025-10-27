use async_stream::stream;
use interprocess::local_socket::tokio::Stream;
use futures_core::stream::Stream as FutureStream;
#[cfg(unix)]
use std::{fs::Permissions, io, os::unix::fs::PermissionsExt, path::Path, pin::Pin};
use std::{io, pin::Pin};
use protos::event::MyEvent;
use protos::event::event_sender_server::EventSenderServer;
use protos::hello::MyGreeter;
use tonic::{
    IntoRequest,
    transport::{Server, server::Connected},
};

use protos::hello::greeter_server::GreeterServer;

// use tokio::fs;
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::{
    fs, io::{AsyncRead, AsyncWrite}, signal
};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};
#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;

struct TonicNamedPipeServer {
    inner: NamedPipeServer,
}

impl TonicNamedPipeServer {
    fn new(np_server: NamedPipeServer) -> Self {
        Self { inner: np_server }
    }
}

impl Connected for TonicNamedPipeServer {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {
        ()
    }
}

impl AsyncRead for TonicNamedPipeServer {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for TonicNamedPipeServer {
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

#[cfg(windows)]
fn create_pipe(name: &str) -> impl FutureStream<Item = io::Result<TonicNamedPipeServer>> {
    let name = format!(r"\\.\pipe\{}", name);

    stream! {
        let mut server = ServerOptions::new().first_pipe_instance(true).create(&name)?;

        loop {
            let client = TonicNamedPipeServer::new(server);

            yield Ok(client);

            server = ServerOptions::new().create(&name)?;
        }
    }
}

#[cfg(unix)]
async fn create_pipe(path: &str) -> Result<UnixListenerStream, io::Error> {
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
    let path = "core-socket";

    let stream = create_pipe(path);

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
