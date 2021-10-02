use std::{any::Any, error::Error, fmt::{Display, Formatter}, net::SocketAddr};
use crossbeam_channel::{SendError};
use laminar::{ErrorKind, Packet};
use tetra::TetraError;

pub type BbResult<T = ()> = Result<T, BbError>;

#[derive(Debug)]
pub enum BbError {
    Tetra(TetraError),
    Laminar(ErrorKind),
    CrossbeamSender(SendError<Packet>),
    CrossbeamReceiver, // Disconnect. Empty is handled by a simple Ok(None)
    Bb(BbErrorType)
}

impl Error for BbError {
}

impl Display for BbError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{:?}", self) // TODO: Propper output formatting
    }
}

pub trait TransformResult<T, E: Error> {
    fn convert(self) -> Result<T, E>;
}

impl<T> TransformResult<T, BbError> for tetra::Result<T> {
    fn convert(self) -> Result<T, BbError> {
        self.or_else(|e| Err(BbError::Tetra(e)))
    }
}

impl<T> TransformResult<T, tetra::TetraError> for BbResult<T> {
    fn convert(self) -> Result<T, tetra::TetraError> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => {
                match e {
                    BbError::Tetra(e) => Err(e),
                    n @ _ => panic!("BbResult unwrapped error: {}", n)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum BbErrorType {
    NetShutdownFailure(Box<dyn Any + Send + 'static>),
    NetNotConnected,
    NetInvalidSender(SocketAddr),
    NetInsufficientAuthority,
    InvalidPlayerID(u16)
}
