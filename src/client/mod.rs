use messages::*;
use connection::Connection;

use openal;

use std::time::Duration;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Client {
    connection: Arc<Connection>,
    active: Arc<Mutex<bool>>
}

impl Client {
    pub fn new() -> Client {
        let connection = Arc::new(Connection::bind_to_random_port());

        Client { connection: connection, active: Arc::new(Mutex::new(true)) }
    }

    pub fn connect(&self, addr: &str) {
        let thread_connection = self.connection.clone();

        println!("Bound to {}", thread_connection.addr());
        println!("Connecting to {}", &addr);

        let listener = openal::listener::default(&Default::default()).unwrap();

        let addr = addr.to_string();
        thread::spawn(move || {
            let message = Message::encoded(MessageType::Connect, None);
            thread_connection.send_to(&message, &addr[..]);

            //let thread_connection = thread_connection.clone();
            thread::spawn(move || {
                let mut capture = openal::capture::default::<i16>(1, 44100, 2048).unwrap();
                loop {
                    capture.start();
                    thread::sleep(Duration::from_millis(50));
                    let samples: Vec<i16> = capture.take().unwrap();

                    let message = Message::encoded(MessageType::Audio, Some(samples));
                    thread_connection.send_to(&message, &addr[..]);
                }
            });
        });

        let thread_connection = self.connection.clone();
        thread::spawn(move || {
            let mut stream = listener.source().unwrap().stream();
            loop {
                let mut buf = [0; 8192];
                let _ = thread_connection.recv_from(&mut buf);

                let message = Message::decoded(&buf);
                if message.message_type == MessageType::Audio {
                    println!("Message: {:?}", message.data.len());
                    stream.push(1, &message.data, 44100).unwrap();
                }

                if stream.state() != openal::source::State::Playing {
                    stream.play();
                }


                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let mut active = self.active.lock().unwrap();
        *active = false;
    }
}
