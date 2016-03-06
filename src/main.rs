extern crate argparse;

use std::net::{UdpSocket};

fn main() {
    let mut address = "".to_string();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Voice communication. Becomes the host if --address is missing.");
        ap.refer(&mut address)
            .add_option(&["--address"], argparse::Store, "Address to connect to");
        ap.parse_args_or_exit();
    }

    if address.len() > 0 {
        let socket = match UdpSocket::bind("0.0.0.0:5513") {
            Ok(s) => s,
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        let buf = [0; 5];
        match socket.send_to(&buf, "0.0.0.0:5514") {
            Ok(size) => println!("Wrote {} bytes", size),
            Err(e) => panic!("unable to send: {}", e)
        };
    } else {
        let socket = match UdpSocket::bind("0.0.0.0:5514") {
            Ok(s) => s,
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        loop {
            // read from the socket
            let mut buf = [0; 10];
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    println!("amt: {}", amt);
                    println!("src: {}", src);
                },
                Err(e) => panic!("unable to receive: {}", e)
            };
        }
    }
}
