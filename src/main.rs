extern crate argparse;
extern crate voicetime;

use voicetime::connection::Connection;

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
        let connection = Connection::new();
        let address = &address[..];
        connection.send_to("tjena".as_bytes(), address);
    } else {
        let connection = Connection::new();
        let local_addr = connection.addr();

        println!("Starting session: {}", local_addr);

        loop {
            let mut buf = [0; 10];
            let (length, addr) = connection.recv_from(&mut buf);
            let bytes_to_string = std::str::from_utf8(&buf[..length]).unwrap();

            println!("{} -> {}", addr, bytes_to_string);
        }
    }
}
