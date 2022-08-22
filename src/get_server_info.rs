use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// Retrieves some information about a server
pub async fn get_server_info(server_address: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(server_address).await?;
    let socket_addr = stream.local_addr()?;

    // write handshake (0x00 packet ID)
    let packet_id = get_var_int(0);
    let protocol_version = get_var_int(-1);
    let server_addr = get_string(&socket_addr.ip().to_string());
    let server_port = socket_addr.port().to_be_bytes();
    let next_state = get_var_int(1);

    let packet = &[
        packet_id.as_slice(),
        protocol_version.as_slice(),
        server_addr.as_slice(),
        &server_port,
        next_state.as_slice(),
    ]
    .concat();

    stream
        .write_all(&[&get_var_int(packet.len() as i32), packet.as_slice()].concat())
        .await?;

    // follow up with status request packet (0x00)
    let status_request = get_var_int(0);
    stream
        .write_all(
            &[
                &get_var_int(status_request.len() as i32),
                status_request.as_slice(),
            ]
            .concat(),
        )
        .await?;

    // read response packet
    let mut buf = [0; 256];
    let res = stream.read(&mut buf).await?;
    println!("Amount read: {res} = {buf:?}");

    Ok(())
}

fn get_var_int(mut param_int: i32) -> Vec<u8> {
    let mut bytes = Vec::new();

    while (param_int & -128) != 0 {
        bytes.push((param_int & 127 | 128) as u8);
        param_int = ((param_int as u32) >> 7) as i32;
    }
    bytes.push(param_int as u8);

    bytes
}

fn get_string(string: &str) -> Vec<u8> {
    let bytes = string.as_bytes();
    [&get_var_int(bytes.len() as i32), bytes].concat()
}
