pub trait Sender {
    fn send_to(self) -> bool;
}

trait Receiver {
    fn recv_from();
}

pub trait Connectable {

}
