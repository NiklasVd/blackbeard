use tetra::{Context, Event, State};
use crate::{Rcc, V2, ui_element::{UIElement, UIReactor, UITransform}};

pub type UIElemBox = Box<dyn UIElement + 'static>;

#[derive(Clone, Copy)]
pub enum UIAlignment {
    Vertical,
    Horizontal
}

pub struct Grid {
    pub alignment: UIAlignment,
    pub transform: UITransform,
    pub elements: Vec<UIElemBox>
}

impl Grid {
    pub fn new(ctx: &mut Context, alignment: UIAlignment, pos: V2, size: V2, padding: f32)
        -> tetra::Result<Grid> {
        Ok(Grid {
            alignment, transform: UITransform::new(ctx, pos, size, V2::one(), padding)?,
            elements: Vec::new()
        })
    }

    pub fn add_element<T: UIElement + 'static>(&mut self, element: T,
        index: usize) {
        assert!(index <= self.elements.len());
        let element_boxed = Box::new(element);
        self.elements.insert(index, element_boxed);
        self.update_element_alignments(index);
    }

    pub fn remove_element(&mut self, index: usize) -> UIElemBox {
        assert!(index <= self.elements.len() - 1);
        self.elements.swap_remove(index)
    }

    fn update_element_alignments(&mut self, from_index: usize) {
        let next_aligned_pos = match from_index {
            0 => self.transform.position,
            _ => self.elements.get(from_index - 1).unwrap()
                .get_transform().get_next_aligned_pos(self.alignment)
        };
        if let Some(element) = self.elements.get_mut(from_index) {
            element.get_transform_mut().position = next_aligned_pos;
            self.update_element_alignments(from_index + 1);
        }
    } 
}

impl UIElement for Grid {
    fn get_name(&self) -> &str {
        "Grid"
    }

    fn get_reactor(&self) -> Option<Rcc<dyn UIReactor>> {
        None
    }

    fn get_transform(&self) -> &UITransform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut UITransform {
        &mut self.transform
    }
}

impl State for Grid {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        for element in self.elements.iter_mut() {
            element.update(ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        for element in self.elements.iter_mut() {
            element.draw(ctx)?;
            element.draw_rect(ctx);
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        for element in self.elements.iter_mut() {
            element.event(ctx, event.clone())?;
        }
        Ok(())
    }
}
