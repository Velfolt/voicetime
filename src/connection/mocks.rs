pub use connection::traits::*;

pub struct SuccessfulSender;

impl Sender for SuccessfulSender {
    fn send_to(self) -> bool {
        true
    }
}
