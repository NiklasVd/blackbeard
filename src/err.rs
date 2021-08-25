use std::{error::Error, fmt::{Display, Formatter}, net::SocketAddr};

use crossbeam_channel::{SendError, TryRecvError};
use laminar::{ErrorKind, Packet};
use tetra::TetraError;

pub type BbResult<T> = Result<T, BbError>;

#[derive(Debug)]
pub enum BbError {
    Tetra(TetraError),
    Laminar(ErrorKind),
    CrossbeamSender(SendError<Packet>),
    CrossbeamReceiver(TryRecvError),
    Bb(BbErrorType)
}

impl Error for BbError {
}

impl Display for BbError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{:?}", self) // TODO: Propper output formatting
    }
}

#[derive(Debug)]
pub enum BbErrorType {
    NetTimeout(SocketAddr),
    NetInvalidSender(SocketAddr),
    InvalidPlayerID(u16),
    NetPollEmpty
}
