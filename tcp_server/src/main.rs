use std::net::{TcpListener, TcpStream};
use std::io;
use threadpool::{self, ThreadPool};
use tcp_utils;

const IP: &'static str = "127.0.0.1";
const PORT: &'static str = "8080";

fn log_read_bytes(stream: &mut TcpStream) -> io::Result<()> {
    let peer_connection = stream.peer_addr()?;
    let recvd_bytes = tcp_utils::read_bytes(stream)?;

    if recvd_bytes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            String::from("`tcp_utils::log_read_bytes` no bytes readed"),
        ));
    }
    println!(
        "[{}:{} |->|] {{ {} bytes }}\n\t`{}`",
        peer_connection.ip(),
        peer_connection.port(),
        recvd_bytes.len(),
        std::str::from_utf8(&recvd_bytes).unwrap(),
    );

    Ok(())
}

fn log_write_bytes(stream: &mut TcpStream, src: &[u8]) -> io::Result<()> {
    let peer_connection = stream.peer_addr()?;

    tcp_utils::send_bytes(stream, src)?;
    println!(
        "[{}:{} |<-|] {{ {} bytes }}\n\t`{}`",
        peer_connection.ip(),
        peer_connection.port(),
        src.len(),
        std::str::from_utf8(src).unwrap(),
    );

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let peer_connection = stream.peer_addr()?;
    println!(
        "[Connection established]\n\tIP_ADDR: {}\n\tPORT   : {}",
        peer_connection.ip(), 
        peer_connection.port()
    );

    let msg = b"Connection with the server successful!";
    log_write_bytes(&mut stream, msg)?;

    // let mut buf = [0; 512];
    loop {
        if let Err(_) = log_read_bytes(&mut stream) {
            println!(
                "[Connection closed]\n\tIP_ADDR: {}\n\tPORT   : {}",
                peer_connection.ip(),
                peer_connection.port()
            );
            break;
        }

        // match stream.read(&mut buf) {
        //     Ok(0) => {
        //         println!(
        //             "[Connection closed]\n\tIP_ADDR: {}\n\tPORT   : {}",
        //             peer_connection.ip(),
        //             peer_connection.port()
        //         );
        //         break;
        //     }
        //     Ok(n) => {
        //         println!(
        //             "[{}:{}(->)] {{ {} bytes }}\n\t`{}`",
        //             peer_connection.ip(),
        //             peer_connection.port(),
        //             n,
        //             std::str::from_utf8(&buf).unwrap(),
        //         );
        //     }
        //     Err(e) => {
        //         println!("[ERROR]:\n\tread from stream: `{}`", e);
        //     }
        // }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", IP, PORT))?;
    let pool = ThreadPool::new(4, threadpool::Mode::Quiet);

    println!("[Server info]\n\tIP_ADDR: {}\n\tPORT   : {}\n", IP, PORT);

    for stream in listener.incoming() {
        let stream = stream?;

        pool.execute(|| {
            if let Err(e) = handle_connection(stream) {
                println!("[HANDLE ERROR]:\n\t{}", e);
            }
        });
    }

    Ok(())
}
