use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::protocol::encoding::VarInt;

/// Retrieves some information about a server
pub async fn get_server_info(server_address: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(server_address).await?;
    let socket_addr = stream.peer_addr()?;

    // write handshake (0x00 packet ID)
    let packet_id: VarInt = 0.into();
    let protocol_version: VarInt = (-1).into();
    let server_addr = get_string(&socket_addr.ip().to_string());
    let server_port = socket_addr.port().to_be_bytes();
    let next_state: VarInt = 1.into();

    let packet = &[
        packet_id.as_slice(),
        protocol_version.as_slice(),
        server_addr.as_slice(),
        &server_port,
        next_state.as_slice(),
    ]
    .concat();

    let packet_len: VarInt = (packet.len() as i32).into();

    stream
        .write_all(&[packet_len.as_slice(), packet.as_slice()].concat())
        .await?;

    // follow up with status request packet (0x00)
    let status_request: VarInt = 0.into();
    let status_request_len: VarInt = (status_request.as_slice().len() as i32).into();
    stream
        .write_all(&[status_request_len.as_slice(), status_request.as_slice()].concat())
        .await?;

    // read response packet
    let mut buf = [0; 256];
    let res = stream.read(&mut buf).await?;
    println!("Amount read: {res} = {buf:?}");

    Ok(())
}

fn get_string(string: &str) -> Vec<u8> {
    let bytes = string.as_bytes();
    let bytes_len: VarInt = (bytes.len() as i32).into();

    [bytes_len.as_slice(), bytes].concat()
}
