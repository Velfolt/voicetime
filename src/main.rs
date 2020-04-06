extern crate argparse;
extern crate openal;
extern crate voicetime;
//extern crate bincode;
//extern crate rustc_serialize;

extern crate chan;
extern crate chan_signal;
extern crate bytes;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio;
extern crate tokio_io;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

use chan_signal::Signal;

use std::io;
use std::str;
use tokio::net::*;
use tokio_io::codec::{Encoder, Decoder};
use tokio::prelude::{Sink, Stream, Future, Async};
use bytes::BytesMut;
use tokio::prelude::task::current;

use std::net::SocketAddr;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Message {
    /*Connect,
    Disconnect,*/
    Audio(usize, Vec<u8>)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct GurkCodec;
use bincode::{serialize, deserialize, Infinite};

/*impl UdpCodec for GurkCodec {
    type In = (SocketAddr, Message);
    type Out = (SocketAddr, Message);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        let message : Message = deserialize(&buf[..]).unwrap();
        Ok((*addr, message))
    }

    fn encode(&mut self, (addr, message): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
        let buf : Vec<u8> = serialize(&message, Infinite).unwrap();

        into.extend(buf);
        addr
    }
}*/

impl Decoder for GurkCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        let message : Message = deserialize(&buf[..]).unwrap();
        Ok(Some(message))
    }
}

impl Encoder for GurkCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, data: Self::Item, buf: &mut BytesMut) -> Result<(), io::Error> {
        let message : Vec<u8> = serialize(&data, Infinite).unwrap();

        buf.extend(message);
        Ok(())
    }
}

extern crate rand;
use rand::Rng;

fn random_port() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range::<u16>(10000, 20000)
}

use std::thread;

trait Audio {
    fn capture(&mut self) -> Vec<u8>;
    fn play(&mut self, buf: Vec<u8>);
}

#[derive(Debug)]
struct OpenALAudio<'a> {
    listener: openal::Listener<'a>,
    stream: openal::source::Stream<'a>,
    capture: openal::Capture<u8>,
}

impl<'a> OpenALAudio<'a> {
    fn new() -> Result<OpenALAudio<'a>, ()> {
        let listener = openal::listener::default(&Default::default()).unwrap();

        println!("{:?}", openal::listener::devices());
        println!("{:?}", openal::capture::devices());

        let stream = listener.source().expect("Unable to get source").stream();
        println!("masdf");
        let capture = openal::capture::default::<u8>(1, 44100, 2048).unwrap();
        println!("masdf");
        //capture.start();
        println!("masdf");

        Ok(OpenALAudio { listener: listener, stream: stream, capture: capture })
    }
}

use std::time::Duration;

impl<'a> Audio for OpenALAudio<'a> {
    fn capture(&mut self) -> Vec<u8> {

        while self.capture.len() == 0 {
            thread::sleep(Duration::from_millis(10));
        }

        let data = self.capture.take().unwrap();
        data
    }

    fn play(&mut self, buf: Vec<u8>) {
        self.stream.push(1, &buf, 44100).unwrap();

        if self.stream.state() != openal::source::State::Playing {
            self.stream.play();
        }
    }
}

impl<'a> Stream for OpenALAudio<'a> {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        //println!("hej");
        self.capture.start();
        //println!("hej");
        if self.capture.len() == 0 {
            //println!("nothing in buffer");
            current().notify();
            return Ok(Async::NotReady);
        }

        //thread::sleep(Duration::from_millis(1000));

        let data = match self.capture.take() {
            Ok(data) => data,
            _ => return Ok(Async::Ready(None))
        };

        self.capture.stop();

        //println!("hej");

        Ok(Async::Ready(Some(data)))
    }
}

fn main() {
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    let mut address = "".to_string();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Voice communication. Becomes the host if --address is missing.");
        ap.refer(&mut address)
            .add_option(&["--address"], argparse::Store, "Address to connect to");
        ap.parse_args_or_exit();
    }

    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let a = UdpSocket::bind(&addr).unwrap();
    println!("{:?}", &a.local_addr());

    let (a_sink, a_stream) = UdpFramed::new(a, GurkCodec).split();

    if address.len() > 0 {
        println!("client running");

        // detta borde gå att förbättra!
        // connect -> addr
        // skicka data till addr -> vänta inte
        // vänta på data från addr -> spela upp ljud
        //
        // hmm, är mitt ljudapi gjort som futures? annars gör så det är det och
        // skicka varje paket som en ström, så klart!
        //
        // konceptet server och klient finns knappt via udp, agera därefter
        // HENRIK!

        let server_addr = address.parse().unwrap();
        let mut count = 0;

        let mut audio = OpenALAudio::new().expect("Audio initialization failed");
        let data = audio
            .map_err(|e| println!("error {:?}", e))
            .for_each(move |data| {
                //let a = UdpSocket::bind(&addr).unwrap();
                //a.send_dgram(data, &server_addr).wait();
                a_sink.send((Message::Audio(0, data), server_addr));
                Ok(())
            });

        //let data = audio.capture();

        //println!("tjena {:?}", data);

        let a = a_sink.send_all(a_stream);

        tokio::run(data.join(a).map(|_| {}).map_err(|e| println!("error = {:?}", e)));

        /*let a = a_sink.send((Message::Audio(0, data), server_addr))
            .and_then(|a_sink| {
                let a_stream = a_stream.map(move |(message, addr)| {
                    match message {
                        Message::Audio(mut count, data) => {
                            audio.play(data);
                        },
                    }

                    let data = audio.capture();
                    count += 1;
                    (Message::Audio(count, data), server_addr)
                });

                a_sink.send_all(a_stream)
            });*/
    } else {
        println!("server running");

        //let mut addrs : Vec<SocketAddr> = vec!();
        //let mut addr_consistency = HashMap::new();

        let a_stream = a_stream.map(move |(message, _addr)| {
            println!("woop {:?}", message);
            (Message::Audio(0, vec!()), addr)
            /*match msg {
                Message::Audio(mut count, data) => {
                    let addr_count = addr_consistency.entry(addr).or_insert(count);
                    if addr_count < &mut count {
                        *addr_count = count;
                        for addr in addrs.iter().filter(|f_addr| &addr != *f_addr) {
                            a_sink.send((*addr, Message::Audio(0, vec!())));
                        }
                        (addr, Message::Audio(0, vec!()))
                    } else {
                        println!("inconsistent! ignoring audio");
                        (addr, Message::Audio(0, vec!()))
                }
                    (Message::Audio(0, vec!()))
                },
                /*Message::Connect => {
                    addrs.push(addr);
                    (addr, Message::Connect)
                },
                Message::Disconnect => {
                    let index = addrs.iter().enumerate().find(|&addr2| addr2.1 == &addr).unwrap().0;
                    addrs.remove(index);
                    addr_consistency.remove(&addr);
                    (addr, Message::Disconnect)
                }*/
            }*/
        });

        let a = a_sink.send_all(a_stream);
        tokio::run(a.map(|_| ()).map_err(|e| println!("error = {:?}", e)));
    }

    println!("Quitting");
}
