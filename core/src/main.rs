use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions, Stream};
use std::io::{BufReader, ErrorKind, Read, Write};

use protos::hello::{HelloRequest, HelloReply};
use prost::Message;

fn serialize_resp(resp: HelloReply) -> Vec<u8> {
    let mut msg_buf = Vec::with_capacity(resp.encoded_len());
    resp.encode(&mut msg_buf).unwrap();

    msg_buf
}

fn deserialize_req(req: &[u8]) -> HelloRequest {
    HelloRequest::decode(req).unwrap()
}

fn main() -> std::io::Result<()> {
    let print_name = "core.socket";
    let name = print_name.to_ns_name::<GenericNamespaced>()?;

    let opts = ListenerOptions::new().name(name);

    let listener = match opts.create_sync() {
        Err(e) if e.kind() == ErrorKind::AddrInUse => {
            eprintln!(
                "Error: could not start server because the socket file is occupied. Please check
                if {print_name} is in use by another process and try again."
            );
            
            return Err(e);
        }
        x => x?,
    };

    println!("Server is running at {print_name}");

    for conn in listener.incoming().filter_map(handle_error) {
        let mut conn = BufReader::new(conn);

        println!("Incoming connection!");

        // First read the length
        let mut proto_len = [0];
        conn.read_exact(&mut proto_len)?;

        println!("{proto_len:?}");

        // Then make a buffer to receive the proto buff message
        let mut buffer = Vec::with_capacity(proto_len[0] as usize);
        println!("{}", buffer.capacity());
        conn.read_exact(&mut buffer)?;
        let req = deserialize_req(&buffer);

        println!("Client requested: {:?}", req);

        let reply = HelloReply {
            message: "Ayy lmao".to_string()
        };
        let buf = serialize_resp(reply);

        let len = buf.encoded_len() as u8;
        conn.get_mut().write_all(&[len])?;
        conn.get_mut().write_all(&buf)?;

        buffer.clear();
    }

    Ok(())
}

fn handle_error(conn: std::io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Incoming connection failed: {e}");
            None
        }
    }
}