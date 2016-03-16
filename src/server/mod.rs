use messages::*;
use connection::Connection;
use std::time::Duration;

use bincode::rustc_serialize::decode;

use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Server {
    connection: Arc<Connection>,
    clients: Arc<Mutex<Vec<String>>>,
}

impl Server {
    pub fn new(port: Option<u16>) -> Server {
        let connection = if let Some(port) = port {
            Arc::new(Connection::bind(port))
        } else {
            Arc::new(Connection::bind_to_random_port())
        };

        Server { connection: connection, clients: Arc::new(Mutex::new(vec!())) }
    }

    pub fn listen(&self) {
        let thread_connection = self.connection.clone();
        let clients = self.clients.clone();

        println!("Listening on {}", thread_connection.addr());

        thread::spawn(move || {
            loop {
                let mut buf = [0; 8192];
                let (_length, addr) = thread_connection.recv_from(&mut buf);

                let mut clients = clients.lock().unwrap();
                let client_len = clients.len();

                let mut exists = false;
                let recv_client = format!("{}",addr);
                for addr in clients.iter() {
                    if addr[..] == recv_client[..] {
                        exists = true;
                    }
                }

                if !exists {
                    clients.push(format!("{}", &addr));
                }

                let received_message : Message = decode(&buf).unwrap();
                if received_message.message_type == MessageType::Connect {
                    println!("Someone connected");
                    continue;
                }

                if client_len < 1 {
                    continue;
                }

                let thread_connection = thread_connection.clone();
                let clients = clients.clone();
                let recv_client = Arc::new(format!("{}", &addr));
                let recv_client = recv_client.clone();
                thread::spawn(move || {
                    for addr in clients.iter() {
                        if addr[..] == recv_client[..] {
                            continue;
                        }

                        thread_connection.send_to(&buf, &addr[..]);
                    }
                });

                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn connected_clients(&self) -> usize {
        self.clients.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use connection::Connection;
    use messages::*;

    use std::thread;

    use std::net::Ipv4Addr;

    use bincode::SizeLimit;
    use bincode::rustc_serialize::{encode, decode};

    #[test]
    fn receive_from_a_new_client_with_zero_current_clients() {
        let server = Server::new(Some(12437));
        server.listen();

        let message = Message::new(MessageType::Audio, Some(vec!(5)));
        let encoded: Vec<u8> = encode(&message, SizeLimit::Infinite).unwrap();

        let client = Connection::bind_to_random_port();
        client.send_to(&encoded, "0.0.0.0:12437");

        thread::sleep_ms(50);

        assert_eq!(1, server.connected_clients());
    }

    #[test]
    fn receive_from_a_new_client_with_one_current_clients() {
        let server = Server::new(Some(23456));
        server.listen();

        let thread1 = thread::spawn(|| {
            let message = Message::new(MessageType::Connect, None);
            let encoded: Vec<u8> = encode(&message, SizeLimit::Infinite).unwrap();

            let client = Connection::bind_to_random_port();
            client.send_to(&encoded, "0.0.0.0:23456");

            let mut buf = [0; 8192];
            let _ = client.recv_from(&mut buf);

            let received_message : Message = decode(&buf).unwrap();

            assert_eq!(10, received_message.data[0]);
        });

        let thread2 = thread::spawn(|| {
            let message = Message::new(MessageType::Audio, Some(vec!(10)));
            let encoded: Vec<u8> = encode(&message, SizeLimit::Infinite).unwrap();

            let client2 = Connection::bind_to_random_port();
            client2.send_to(&encoded, "0.0.0.0:23456");
        });

        thread::sleep_ms(100);
        thread2.join().unwrap();
        thread::sleep_ms(100);
        thread1.join().unwrap();

        assert_eq!(2, server.connected_clients());
    }

    #[test]
    fn receive_from_a_new_client_with_two_current_clients() {
        let server = Server::new(Some(24351));
        server.listen();

        let thread1 = thread::spawn(|| {
            let message = Message::new(MessageType::Connect, None);
            let encoded: Vec<u8> = encode(&message, SizeLimit::Infinite).unwrap();

            let client = Connection::bind_to_random_port();
            client.send_to(&encoded, "0.0.0.0:24351");

            let mut buf = [0; 8192];
            let _ = client.recv_from(&mut buf);

            let received_message : Message = decode(&buf).unwrap();

            assert_eq!(10, received_message.data[0]);
        });

        let thread2 = thread::spawn(|| {
            let message = Message::new(MessageType::Audio, None);
            let encoded: Vec<u8> = encode(&message, SizeLimit::Infinite).unwrap();

            let client = Connection::bind_to_random_port();
            client.send_to(&encoded, "0.0.0.0:24351");

            let mut buf = [0; 8192];
            let _ = client.recv_from(&mut buf);

            let received_message : Message = decode(&buf).unwrap();

            assert_eq!(10, received_message.data[0]);
        });

        let thread3 = thread::spawn(|| {
            let message = Message::new(MessageType::Audio, Some(vec!(10)));
            let encoded: Vec<u8> = encode(&message, SizeLimit::Infinite).unwrap();

            let client2 = Connection::bind_to_random_port();
            client2.send_to(&encoded, "0.0.0.0:24351");
        });

        thread::sleep_ms(100);
        thread3.join().unwrap();
        thread::sleep_ms(100);
        thread1.join().unwrap();
        thread::sleep_ms(100);
        thread2.join().unwrap();

        assert_eq!(3, server.connected_clients());
    }

}
