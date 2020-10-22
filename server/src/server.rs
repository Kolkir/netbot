use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::rc::Rc;

use super::message;
use message::SendMessage;

#[derive(Debug)]
enum ServerErrors {
    MissedConnection,
}
impl fmt::Display for ServerErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
impl Error for ServerErrors {}

pub struct Server {
    tcp_stream: Option<TcpStream>,
}

impl Clone for Server {
    fn clone(&self) -> Server {
        match &self.tcp_stream {
            Some(stream) => Server {
                tcp_stream: Some(stream.try_clone().expect("Failed to clone tcp_stream")),
            },
            None => Server::new(),
        }
    }
}

impl Server {
    pub fn new() -> Server {
        Server { tcp_stream: None }
    }
    pub fn wait_client(&mut self, addr: Ipv4Addr, port: u16) -> Result<(), Box<dyn Error>> {
        let socket_addr = SocketAddrV4::new(addr, port);
        let listener = Rc::new(TcpListener::bind(socket_addr)?);
        let port = listener.local_addr()?;
        println!("Listening on {}, access this port from a client", port);
        let (tcp_stream, client_addr) = listener.accept()?; //block  until requested
        self.tcp_stream = Some(tcp_stream);
        println!("Connection received! {:?}", client_addr);
        Ok(())
    }

    pub fn send(&mut self, msg: Box<dyn SendMessage>) -> Result<(), Box<dyn Error>> {
        let mut msg = msg;
        match &mut self.tcp_stream {
            Some(stream) => {
                let mut id_size: [u8; 5] = [0; 5];
                id_size[0] = msg.id();
                let size_bytes = msg.size().to_be_bytes();
                id_size[1..].copy_from_slice(&size_bytes);
                stream.write_all(&id_size)?;
                let bytes = msg.to_bytes();
                match bytes {
                    Some(bytes) => {
                        stream.write_all(bytes)?;
                        return Ok(());
                    }
                    None => return Ok(()),
                }
            }
            None => Err(Box::new(ServerErrors::MissedConnection)),
        }
    }

    pub fn recv(&mut self) -> Result<(u8, Vec<u8>), Box<dyn Error>> {
        match &mut self.tcp_stream {
            Some(stream) => {
                let mut id_size: [u8; 5] = [0; 5];
                stream.read_exact(&mut id_size)?;
                let tmp = slice_as_array!(&id_size[1..], [u8; 4])
                    .expect("Server::recv wrong header data");
                let size = u32::from_be_bytes(*tmp);
                let id = id_size[0];
                let mut buf: Vec<u8> = Vec::new();
                buf.resize(size as usize, 0);
                stream.read_exact(&mut buf)?;
                Ok((id, buf))
            }
            None => Err(Box::new(ServerErrors::MissedConnection)),
        }
    }
}
