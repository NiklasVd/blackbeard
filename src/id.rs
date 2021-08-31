use std::fmt;
use binary_stream::{BinaryStream, Serializable};
use tetra::graphics::Color;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct ID {
    pub name: String,
    pub n: u16
}

impl ID {
    pub fn new(name: String, n: u16) -> ID {
        ID {
            name, n // TODO: ID-generator
        }
    }

    pub fn get_id_color(&self) -> Color {
        Color::WHITE
    }
}

impl Serializable for ID {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_string(&self.name).unwrap();
        stream.write_u16(self.n).unwrap();
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        let name = stream.read_string().unwrap();
        let n = stream.read_u16().unwrap();
        ID::new(name, n)
    }
}

impl fmt::Debug for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}^{}", self.name, self.n)
    }
}
