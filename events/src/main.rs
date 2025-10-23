use interprocess::local_socket::{GenericNamespaced, Stream, prelude::*};
use std::{
    io::{BufReader, prelude::*},
    str::FromStr,
};

use prost::Message;
use protos::hello::{HelloReply, HelloRequest};

fn serialize_req(req: HelloRequest) -> Vec<u8> {
    let mut msg_buf = Vec::with_capacity(req.encoded_len());
    req.encode(&mut msg_buf).unwrap();

    msg_buf
}

fn deserialize_resp(resp: &[u8]) -> HelloReply {
    HelloReply::decode(resp).unwrap()
}

fn main() -> std::io::Result<()> {
    let conn = Stream::connect("core.socket".to_ns_name::<GenericNamespaced>()?)?;
    let mut conn = BufReader::new(conn);

    let msg = HelloRequest {
        name: String::from_str("Butt").unwrap(),
    };

    println!("{msg:?}");

    let req = serialize_req(msg);

    // First get the length of the buffer and send the request
    let len = req.encoded_len() as u8;
    println!("{len}");
    conn.get_mut().write_all(&[len])?;
    conn.get_mut().write_all(&req)?;

    let mut proto_len = [0];
    conn.read_exact(&mut proto_len)?;

    println!("{proto_len:?}");
    let mut msg_buf = Vec::with_capacity(proto_len[0] as usize);
    conn.read_exact(&mut msg_buf)?;

    let reply = deserialize_resp(&msg_buf);

    println!("Server answered: {:?}", reply);
    Ok(())
}
