use tetra::Context;
use crate::{GC, Rcc, button::DefaultButton, grid::Grid, textbox::Textbox};

pub struct Chat {
    grid: Rcc<Grid>,
    messages: Rcc<Grid>,
    msg_txt: Rcc<Textbox>,
    send_button: Rcc<DefaultButton>,
    game: GC
}

// impl Chat {
//     pub fn new(ctx: &mut Context, game: GC) -> Chat {
//         let grid = Grid
//         Chat {

//         }
//     }
// }
