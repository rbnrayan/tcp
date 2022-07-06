use std::net::TcpStream;
use std::io;
use tcp_utils;

fn main() -> io::Result<()> {
    match TcpStream::connect("127.0.0.1:8080") {
        Ok(mut stream) => {
            let bytes_recvd = tcp_utils::read_bytes(&mut stream)?;
            println!("Received: {}", std::string::String::from_utf8(bytes_recvd).unwrap());

            tcp_utils::send_bytes(&mut stream, b"test")?;
        },
        Err(_) => {
            println!("Couldn't conect to the server");
        }
    }
    Ok(())
}
