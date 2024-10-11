use crate::args::Args;
use crate::command::execute_command;
use anyhow::{bail, Result};
use log::{error, info};
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    net::{Ipv4Addr, Ipv6Addr, TcpListener, TcpStream, UdpSocket},
    time::Duration,
};

pub fn is_valid_address(address: &str, version: &u8) -> bool {
    match version {
        4 => address
            .parse::<Ipv4Addr>()
            .map_or(false, |ip| ip.is_global()),
        6 => address
            .parse::<Ipv6Addr>()
            .map_or(false, |ip| ip.is_global()),
        _ => false,
    }
}

fn handle_tcp_connection(mut stream: TcpStream, args: &Args, timeout: Duration) -> Result<()> {
    stream.set_read_timeout(Some(timeout))?;
    if let Some(command) = &args.exec {
        execute_command(stream, command)?;
    } else {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        if let Some(file_path) = &args.file {
            let mut file = File::create(file_path)?;
            file.write_all(&buffer)?;
        } else {
            io::stdout().write_all(&buffer)?;
        }
    }
    Ok(())
}

fn run_tcp_server(args: &Args, destination: String, timeout: Duration) -> Result<()> {
    let listener = TcpListener::bind(destination.clone())?;
    info!("Listening on {}...", destination);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_tcp_connection(stream, args, timeout) {
                    error!("Failed to handle connection: {}", e);
                }
            }
            Err(e) => error!("Failed to accept connection: {}", e),
        }
    }
    Ok(())
}

fn handle_udp_connection(socket: UdpSocket, args: &Args, timeout: Duration) -> Result<()> {
    let mut buffer = vec![0u8; 65535];
    let (amt, _src) = socket.recv_from(&mut buffer)?;
    socket.set_read_timeout(Some(timeout))?;
    buffer.truncate(amt);

    if let Some(command) = &args.exec {
        execute_command(io::Cursor::new(buffer), command)?;
    } else {
        if let Some(file_path) = &args.file {
            let mut file = File::create(file_path)?;
            file.write_all(&buffer)?;
        } else {
            io::stdout().write_all(&buffer)?;
        }
    }
    Ok(())
}

fn run_udp_server(args: &Args, destination: String, timeout: Duration) -> Result<()> {
    let socket = UdpSocket::bind(destination.clone())?;
    info!("Listening on {}...", destination);
    handle_udp_connection(socket, args, timeout)
}

pub fn run_server(args: &Args, protocol: &str, timeout: Duration) -> Result<()> {
    let address = args.address.as_ref().unwrap();
    let port = args.port.unwrap();
    let destination = format!("{}:{}", address, port);

    match protocol {
        "tcp" => run_tcp_server(args, destination, timeout),
        "udp" => run_udp_server(args, destination, timeout),
        _ => bail!("Invalid protocol '{}'.", protocol.to_uppercase()),
    }
}

fn prepare_buffer_from_file_or_stdin(args: &Args) -> Result<Vec<u8>> {
    if let Some(file_path) = &args.file {
        let file = File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);
        let mut buffer = Vec::new();
        buf_reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    } else {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

fn run_tcp_client(destination: String, buffer: Vec<u8>, timeout: Duration) -> Result<()> {
    let mut stream = TcpStream::connect(destination)?;
    stream.set_write_timeout(Some(timeout))?;
    stream.write_all(&buffer)?;
    Ok(())
}

fn run_udp_client(destination: String, buffer: Vec<u8>, timeout: Duration) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_write_timeout(Some(timeout))?;
    socket.send_to(&buffer, destination)?;
    Ok(())
}

pub fn run_client(args: &Args, protocol: &str, timeout: Duration) -> Result<()> {
    let address = args.address.as_ref().unwrap();
    let port = args.port.unwrap();
    let destination = format!("{}:{}", address, port);

    let buffer = prepare_buffer_from_file_or_stdin(args)?;

    match protocol {
        "tcp" => run_tcp_client(destination, buffer, timeout),
        "udp" => run_udp_client(destination, buffer, timeout),
        _ => bail!("Invalid protocol '{}'.", protocol.to_uppercase()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use clap::Parser;

    #[test]
    fn test_is_valid_address() {
        assert!(is_valid_address("192.168.0.1", &4));
        assert!(!is_valid_address("999.999.999.999", &4));
        assert!(is_valid_address("::1", &6));
        assert!(!is_valid_address("invalid-ip", &6));
    }

    #[test]
    fn test_tcp_communication() {
        let server = thread::spawn(|| {
            let args = Args::parse_from(&["test", "--listen", "127.0.0.1", "8080"]);
            run_server(&args, "tcp", Duration::from_secs(5)).unwrap();
        });

        thread::sleep(Duration::from_secs(1));

        let client = thread::spawn(|| {
            let args = Args::parse_from(&["test", "127.0.0.1", "8080"]);
            run_client(&args, "tcp", Duration::from_secs(5)).unwrap();
        });

        server.join().unwrap();
        client.join().unwrap();
    }

    #[test]
    fn test_udp_communication() {
        let server = thread::spawn(|| {
            let args =
                Args::parse_from(&["test", "--listen", "--protocol", "UDP", "127.0.0.1", "8080"]);
            run_server(&args, "udp", Duration::from_secs(5)).unwrap();
        });

        thread::sleep(Duration::from_secs(1));

        let client = thread::spawn(|| {
            let args = Args::parse_from(&["test", "--protocol", "UDP", "127.0.0.1", "8080"]);
            run_client(&args, "udp", Duration::from_secs(5)).unwrap();
        });

        server.join().unwrap();
        client.join().unwrap();
    }
}
