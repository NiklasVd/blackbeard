use tetra::{Context};
use crate::{GC, Rcc, V2, button::{Button, DefaultButton}, grid::{Grid, UIAlignment, UILayout}, label::{FontSize, Label}, textbox::Textbox, ui_element::{DefaultUIReactor}};

const MAX_CHAT_MESSAGES_COUNT: usize = 13;

pub struct Chat {
    grid: Rcc<Grid>,
    messages_grid: Rcc<Grid>,
    msg_txt: Rcc<Textbox>,
    send_button: Rcc<DefaultButton>,
    game: GC
}

impl Chat {
    pub fn new(ctx: &mut Context, layout: UILayout, grid: &mut Grid, game: GC)
        -> tetra::Result<Chat> {
        let mut chat_grid = Grid::new(ctx, UIAlignment::Vertical, layout,
            V2::new(300.0, 285.0), 1.0)?;
        let messages_grid = chat_grid.add_element(Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::new(300.0, 200.0), 1.0)?);
        
        let mut text_grid = Grid::default(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::new(300.0, 35.0), 1.0)?;
        let msg_txt = text_grid.add_element(Textbox::new(ctx, "", V2::new(250.0, 35.0),
            1.0, game.clone())?);
        let send_button = text_grid.add_element(Button::new(ctx, "Send",
            V2::new(50.0, 35.0), 1.0, DefaultUIReactor::new(), game.clone())?);
        chat_grid.add_element(text_grid);
        let grid = grid.add_element(chat_grid);

        Ok(Chat {
            grid, messages_grid, msg_txt, send_button, game
        })
    }

    pub fn add_line(&mut self, ctx: &mut Context, text: &str) -> tetra::Result {
        let mut messages_grid_ref = self.messages_grid.borrow_mut();
        let msg_count = messages_grid_ref.elements.len();
        if msg_count >= MAX_CHAT_MESSAGES_COUNT {
            messages_grid_ref.remove_element_at(0);
        }
        messages_grid_ref.add_element(Label::new(ctx,
            text, FontSize::Small, 2.0, self.game.clone())?);
        Ok(())
    }

    pub fn add_message(&mut self, ctx: &mut Context, sender: &str, msg: &str)
        -> tetra::Result {
        self.add_line(ctx, &format!("{}: {}", sender, msg))
    }

    pub fn is_focused(&self) -> bool {
        self.msg_txt.borrow().is_focused()
    }

    pub fn check_messages(&mut self, ctx: &mut Context) -> Option<String> {
        let mut msg_txt_ref = self.msg_txt.borrow_mut();
        let msg = msg_txt_ref.get_text().to_owned();
        if !msg.is_empty() &&
            (self.send_button.borrow().is_pressed() || msg_txt_ref.confirm_enter(ctx)) {
            msg_txt_ref.set_text("");
            Some(msg.to_owned())
        } else {
            None
        }
    }
}
