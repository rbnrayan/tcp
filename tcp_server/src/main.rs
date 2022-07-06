use std::net::{TcpListener, TcpStream};
use std::io;
use threadpool::{self, ThreadPool};
use tcp_utils;

const IP: &'static str = "127.0.0.1";
const PORT: &'static str = "8080";

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let peer_connection = stream.peer_addr()?;
    let read_logfn = |recvd_bytes: &Vec<u8>| println!(
        "[{}:{} |>>|] {{ {} bytes }} :\n\t`{}`",
        peer_connection.ip(),
        peer_connection.port(),
        recvd_bytes.len(),
        std::str::from_utf8(&recvd_bytes).unwrap(),
    );
    let send_logfn = |src: &[u8]| println!(
        "[{}:{} |<<|] {{ {} bytes }} :\n\t`{}`",
        peer_connection.ip(),
        peer_connection.port(),
        src.len(),
        std::str::from_utf8(src).unwrap(),
    );

    println!(
        "[Connection established]\n\tIP_ADDR: {}\n\tPORT   : {}",
        peer_connection.ip(), 
        peer_connection.port()
    );

    let msg = b"Connection with the server successful!";
    tcp_utils::log_send_bytes(&mut stream, msg, send_logfn)?;

    loop {
        let read_bytes_res = tcp_utils::log_read_bytes(&mut stream, read_logfn);
        if let Err(_) = read_bytes_res {
            println!(
                "[Connection closed]\n\tIP_ADDR: {}\n\tPORT   : {}",
                peer_connection.ip(),
                peer_connection.port()
            );
            break;
        }
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
