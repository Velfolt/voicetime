extern crate rand;

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
        let socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();

        Connection { socket: socket }
    }

    pub fn send_to<T: ToSocketAddrs>(&self, buf: &[u8], addr: T) -> usize {
        self.socket.send_to(buf, addr).unwrap()
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> (usize, SocketAddr) {
        self.socket.recv_from(buf).unwrap()
    }

    pub fn addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    use std::str;

    #[test]
    fn random_port_generates_in_range_of_10000_20000() {
        let port = random_port();

        assert!(port >= 10000);
        assert!(port <= 20000);
    }

    #[test]
    fn receive_from_another_socket() {
        let listener = Connection::new();
        let addr = listener.addr();

        let thread = thread::spawn(move || {
            let mut buf = [0; 10];
            let (length, _) = listener.recv_from(&mut buf);

            (length, buf)
        });

        let connection = Connection::new();
        let buf = "hejsan".as_bytes();
        assert_eq!(6, connection.send_to(&buf, &addr));

        let (length, received) = thread.join().unwrap();
        assert_eq!(6, length);
        let bytes_to_string = str::from_utf8(&received[..length]).unwrap();
        assert_eq!("hejsan", bytes_to_string);
    }
}
