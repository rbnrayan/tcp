use std::net::{TcpStream, IpAddr};
use std::io;
use std::env;
use crate::app::App;

mod app;

fn main() -> io::Result<()> {
    if env::args().len() < 2 {
        println!("Usage: tcp_client <ip_address:port>");
        return Ok(())
    }

    let args = env::args().collect::<Vec<String>>();
    let addr = &args[1];

    match TcpStream::connect(addr) {
        Ok(stream) => {
            let send_logfn = |src: &[u8], ip: IpAddr, port: u16| println!(
                "[{}:{} |>>|] {{ {} bytes }} : `{}`",
                ip,
                port,
                src.len(),
                std::str::from_utf8(src).unwrap(),
            );

            let mut app = App::new(stream, Some(send_logfn));
            app.logs(true);
            app.run()?
        },
        Err(_) => {
            println!("Couldn't conect to the server");
        }
    }
    Ok(())
}
