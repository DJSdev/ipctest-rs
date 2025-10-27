use std::io;
use futures_core::stream::Stream as FutureStream;

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
use crate::pipe::server::windows::TonicNamedPipeServer;

#[cfg(unix)]
pub async fn create_pipe(pipe_name: &str) -> Result<UnixListenerStream, io::Error> {
    use std::path::Path;
    use std::fs::Permissions;
    use tokio::fs;
    use tokio::net::UnixListener;
    use tokio_stream::wrappers::UnixListenerStream;
    use std::os::unix::fs::PermissionsExt;

    let path = format!("/tmp/{}", pipe_name);

    if fs::metadata(&path).await.is_ok() {
        fs::remove_file(&path).await?;
    }

    std::fs::create_dir_all(Path::new(&path).parent().unwrap())?;

    let uds = UnixListener::bind(&path).unwrap();
    fs::set_permissions(&path, Permissions::from_mode(0o600)).await?;

    Ok(UnixListenerStream::new(uds))
}

#[cfg(windows)]
pub async fn create_pipe(pipe_name: &str) -> impl FutureStream<Item = io::Result<TonicNamedPipeServer>> {
    use async_stream::stream;
    use tokio::net::windows::named_pipe::ServerOptions;

    let name = format!(r"\\.\pipe\{}", pipe_name);

    stream! {
        let mut server = ServerOptions::new().first_pipe_instance(true).create(&name)?;

        loop {
            let client = TonicNamedPipeServer::new(server);

            yield Ok(client);

            server = ServerOptions::new().create(&name)?;
        }
    }
}