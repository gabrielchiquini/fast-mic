use ringbuf::Producer;

use std::{
    io::Read,
    net::{Shutdown, SocketAddr, TcpStream},
    time::Duration,
};

const BUFFER_SIZE: usize = 3840;

use anyhow::{format_err, Result};

pub fn socket_connect(
    address: &str,
    media_producer: Producer<i16>,
) -> Result<SocketState, SocketError> {
    let address_parsed = (address)
        .parse::<SocketAddr>()
        .map_err(|_| SocketError::AddressError)?;

    let stream = TcpStream::connect(address_parsed).map_err(|err| {
        eprintln!("Error connecting: {}", err);
        SocketError::ConnectionError
    })?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|_| SocketError::SetupError)?;

    Ok(SocketState {
        address: address.to_owned(),
        stream,
        media_producer,
        buffer: [0u8; BUFFER_SIZE],
    })
}

pub struct SocketState {
    pub address: String,
    stream: TcpStream,
    media_producer: Producer<i16>,
    buffer: [u8; BUFFER_SIZE],
}

impl SocketState {
    pub fn seek(&mut self) -> Result<()> {
        let media_producer = &mut self.media_producer;
        for _ in 0..300 {
            // avoid leaving function context
            match (&mut self.stream).read_exact(&mut self.buffer) {
                Ok(_) => {
                    let current_data = &self.buffer[0..BUFFER_SIZE];
                    let mut last_sample = 0_i16;
                    for i in (0..current_data.len()).step_by(2) {
                        let raw_value = i16::from_le_bytes([current_data[i], current_data[i + 1]]);
                        let sample: i32 = raw_value as i32 + last_sample as i32;
                        last_sample = (sample / 2) as i16;
                        if media_producer.is_full() {
                            eprintln!("Media producer full");
                            break;
                        }
                        if media_producer.push(last_sample) == Err(last_sample) {
                            eprintln!("Can't push item: {}", last_sample);
                        };
                    }
                }
                Err(err) => {
                    eprintln!("Error seeking {:#?}", err);
                    return Err(format_err!("Connection lost"));
                }
            }
        }
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.stream
            .shutdown(Shutdown::Both)
            .map_err(|err| format_err!("Error shutting down socket: {}", err))
    }
}

#[derive(Debug)]
pub enum SocketError {
    AddressError,
    ConnectionError,
    SetupError,
}
