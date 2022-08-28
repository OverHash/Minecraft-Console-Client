use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::protocol::{
    packets::{Handshake, Status},
    Packet,
};

/// Retrieves some information about a server
pub async fn get_server_info(server_address: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(server_address).await?;
    let socket_addr = stream.peer_addr()?;

    // write handshake
    // protocol_version set to `-1` is the convention when pinging
    let packet: Packet =
        Handshake::new(-1, socket_addr.ip().to_string(), socket_addr.port(), true)?.into();
    stream.write_all(&Vec::try_from(packet)?).await?;

    // follow up with status request packet (0x00)
    let packet: Packet = Status::default().into();
    stream.write_all(&Vec::try_from(packet)?).await?;

    // read response packet
    let mut buf = [0; 256];
    let res = stream.read(&mut buf).await?;
    println!("Amount read: {res} = {buf:?}");

    Ok(())
}
