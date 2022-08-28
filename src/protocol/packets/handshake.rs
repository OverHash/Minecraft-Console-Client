use std::num::TryFromIntError;

use crate::protocol::{
    encoding::{EncodedString, VarInt},
    Packet,
};

pub struct Handshake {
    /// The version of the client protocol.
    protocol_version: VarInt,
    /// The address of the server to connect to (e.g., "localhost").
    server_address: EncodedString,
    /// The port of the server to connect to (e.g., 25565).
    server_port: [u8; 2],
    /// The next state for the request.
    next_state: VarInt,
}

impl Handshake {
    /// Creates a new Handshake packet, given the `protocol_version` of the client, the
    /// `server_address` to connect to, the `server_port` of the server, and if the next request is
    /// a status request.
    pub fn new(
        protocol_version: i32,
        server_address: String,
        server_port: u16,
        is_status: bool,
    ) -> Result<Self, TryFromIntError> {
        Ok(Self {
            protocol_version: VarInt::from(protocol_version),
            server_address: server_address.try_into()?,
            server_port: server_port.to_be_bytes(),
            next_state: VarInt::from(if is_status { 1 } else { 2 }),
        })
    }
}

/// Implement conversion from Handshake -> Packet
impl From<Handshake> for Packet {
    fn from(p: Handshake) -> Self {
        Self::new(
            0x00,
            vec![
                p.protocol_version.as_slice(),
                &p.server_address.as_slice(),
                &p.server_port,
                p.next_state.as_slice(),
            ]
            .concat(),
        )
    }
}
