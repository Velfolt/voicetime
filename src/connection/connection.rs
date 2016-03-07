extern crate rand;

use connection::traits::*;

use std::net::UdpSocket;
use std::net::ToSocketAddrs;
use std::net::SocketAddr;
use self::rand::Rng;

pub fn random_port() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range::<u16>(10000, 20000)
}

pub struct Connection {
    socket: UdpSocket
}

impl Connection {
    pub fn new() -> Connection {
        let port = random_port();

        let socket = match UdpSocket::bind(("0.0.0.0", port)) {
            Ok(s) => s,
            Err(e) => panic!("Unable to bind socket: {}", e)
        };

        Connection { socket: socket }
    }

    pub fn send_to<T: ToSocketAddrs>(&self, buf: &[u8], addr: T) {
        self.socket.send_to(buf, addr);
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> (usize, SocketAddr) {
        match self.socket.recv_from(buf) {
            Ok((length, addr)) => {
                (length, addr)
            },
            Err(e) => panic!("Unable to receive: {}", e)
        }
    }

    pub fn addr(&self) -> SocketAddr {
        match self.socket.local_addr() {
            Ok(a) => a,
            Err(e) => panic!("No local addr: {}", e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    //use connection::mocks::*;
    use std::str;

    #[test]
    fn random_port_generates_in_range_of_10000_20000() {
        let port = random_port();

        assert!(port >= 5000);
        assert!(port <= 20000);
    }

    #[test]
    fn receive_from_self() {
        let listener = Connection::new();
        let addr = listener.addr();

        let child = thread::spawn(move || {
            let mut buf = [0; 10];
            let (l, _) = listener.recv_from(&mut buf);

            (l, buf)
        });

        let connection = Connection::new();
        let buf = "hejsan".as_bytes();
        connection.send_to(&buf, &addr);

        // some work here
        let (length, received) = match child.join() {
            Ok(a) => a,
            Err(_e) => panic!()
        };

        let bytes_to_string = str::from_utf8(&received[..length]).unwrap();
        assert_eq!("hejsan", bytes_to_string);
    }
}
