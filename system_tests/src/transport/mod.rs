pub mod grpc;
pub mod tcp;

use faction::PeerId;
use faction::command::Command;

pub trait Transport: Send {
    fn send(&mut self, to: PeerId, message: Command);

    fn recv(&mut self) -> Option<(PeerId, Command)>;
}
