extern crate argparse;
extern crate openal;
extern crate voicetime;
extern crate bincode;
extern crate rustc_serialize;
#[macro_use]
extern crate chan;
extern crate chan_signal;

use chan_signal::Signal;

use voicetime::server::Server;
use voicetime::client::Client;

fn main() {
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    let mut port = 0;
    let mut address = "".to_string();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Voice communication. Becomes the host if --address is missing.");
        ap.refer(&mut port)
            .add_option(&["--port"], argparse::Store, "Port to bind to");
        ap.refer(&mut address)
            .add_option(&["--address"], argparse::Store, "Address to connect to");
        ap.parse_args_or_exit();
    }

    if address.len() > 0 {
        let client = Client::new();
        client.connect(&address);
    } else {
        let server = if port != 0 {
            Server::new(Some(port))
        } else {
            Server::new(None)
        };

        server.listen();
    }

    chan_select! {
        signal.recv() -> _signal => {
            println!("Shutting down.")
        }
    }

}
