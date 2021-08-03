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
