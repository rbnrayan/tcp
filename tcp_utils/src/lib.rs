use std::net::TcpStream;
use std::io::{self, Read, Write};

pub fn send_bytes(stream: &mut TcpStream, src: &[u8]) -> io::Result<()> {
    let mut data = Vec::from(src);
    data.push(b'\0');

    let bytes_written = stream.write(&data)?;

    if data.len() != bytes_written {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "`tcp_utils::send_bytes`: expected {} bytes to be written, found {}",
                data.len(),
                bytes_written
            )
        ))
    }
    Ok(())
}

pub fn read_bytes(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    // 128 bytes seems ok (for me) in this case
    let mut buf = [0; 128];
    let mut bytes = Vec::new();
    loop {
        match stream.read(&mut buf) {
            Ok(_) => {
                if buf.iter().all(|&b| b != b'\0') {
                    bytes.extend_from_slice(&buf);
                } else {
                    let truncated_buf = buf.into_iter()
                        .take_while(|&b| b != b'\0')
                        .collect::<Vec<u8>>();
                    bytes.extend_from_slice(&truncated_buf[..]);
                    break;
                }
            }
            Err(e) => return Err(
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("`tcp_utils::send_bytes`: {}", e),
                )
            ),
        }
    }
    Ok(bytes)
}

