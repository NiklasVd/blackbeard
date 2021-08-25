use tetra::{Context, Event};
use crate::{Rcc, V2, ui_element::{UIElement}, ui_transform::UITransform, wrap_rcc};

pub type UIRcc = Rcc<dyn UIElement>;

#[derive(Clone, Copy)]
pub enum UIAlignment {
    Vertical,
    Horizontal
}

pub struct Grid {
    pub alignment: UIAlignment,
    pub transform: UITransform,
    pub elements: Vec<UIRcc>
}

impl Grid {
    pub fn new(ctx: &mut Context, alignment: UIAlignment, pos: V2, size: V2, padding: f32)
        -> tetra::Result<Grid> {
        Ok(Grid {
            alignment, transform: UITransform::new(ctx, pos, size, V2::one(), padding)?,
            elements: Vec::new()
        })
    }

    pub fn add_element_at<T: UIElement + 'static>(&mut self, element: T,
        index: usize) -> Rcc<T> {
        assert!(index <= self.elements.len());
        let element_rcc = wrap_rcc(element);
        self.elements.insert(index, element_rcc.clone());
        self.update_element_alignments(index);
        element_rcc
    }

    pub fn add_element<T: UIElement + 'static>(&mut self, element: T) -> Rcc<T> {
        self.add_element_at(element, self.elements.len())
    }

    pub fn remove_element_at(&mut self, index: usize) -> UIRcc {
        assert!(index <= self.elements.len() - 1);
        self.elements.swap_remove(index)
    }

    fn update_element_alignments(&mut self, from_index: usize) {
        let next_aligned_pos = match from_index {
            0 => V2::zero(),
            _ => self.elements[from_index - 1].borrow()
                .get_transform().get_next_aligned_pos(self.alignment)
        };
        if let Some(element) = self.elements.get(from_index) {
            element.borrow_mut().get_transform_mut().position = next_aligned_pos;
            self.update_element_alignments(from_index + 1);
        }
    } 
}

impl UIElement for Grid {
    fn get_name(&self) -> &str {
        "Grid"
    }

    fn get_transform(&self) -> &UITransform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut UITransform {
        &mut self.transform
    }

    fn update_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        let abs_pos = self.transform.get_abs_pos(parent_pos);
        for element in self.elements.iter() {
            let mut element_ref = element.borrow_mut();
            element_ref.update_element(ctx, abs_pos)?;
            element_ref.update_reactor(ctx, abs_pos)?;
        }
        Ok(())
    }

    fn draw_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        let abs_pos = self.transform.get_abs_pos(parent_pos);
        for element in self.elements.iter() {
            let mut element_ref = element.borrow_mut();
            element_ref.draw_element(ctx, abs_pos)?;
            element_ref.draw_rect(ctx, abs_pos);
        }
        Ok(())
    }

    fn event_element(&mut self, ctx: &mut Context, event: Event, parent_pos: V2)
        -> tetra::Result {
        let abs_pos = self.transform.get_abs_pos(parent_pos);
        for element in self.elements.iter() {
            element.borrow_mut().event_element(ctx, event.clone(), parent_pos)?;
        }
        Ok(())
    }
}
