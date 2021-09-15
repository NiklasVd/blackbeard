use tetra::{Context, Event, window::{get_height, get_width}};
use crate::{GC, Rcc, V2, image::Image, ui_element::{DefaultUIReactor, UIElement, UIReactor}, ui_transform::UITransform, wrap_rcc};

pub type UIRcc = Rcc<dyn UIElement>;

#[derive(Clone, Copy)]
pub enum UIAlignment {
    Vertical,
    Horizontal
}

pub enum UILayout {
    Default,
    BottomLeft,
    Centre,
    TopRight,
    BottomRight
}

pub struct Grid {
    pub transform: UITransform,
    pub elements: Vec<UIRcc>,
    alignment: UIAlignment,
    layout: UILayout,
    background: Option<Image>,
    reactor: DefaultUIReactor
}

impl Grid {
    pub fn new(ctx: &mut Context, alignment: UIAlignment, layout: UILayout,
        size: V2, padding: f32) -> tetra::Result<Grid> {
        Self::new_bg(ctx, alignment, layout, size, padding, None, None)
    }

    pub fn new_bg(ctx: &mut Context, alignment: UIAlignment, layout: UILayout,
        size: V2, padding: f32, background: Option<String>, game: Option<GC>)
        -> tetra::Result<Grid> {
        let background = match background {
            Some(background) => Some(Image::new(ctx, size, padding, background,
                true, game.unwrap())?),
            _ => None
        };
        Ok(Grid {
            alignment, layout, transform: UITransform::new(ctx, V2::zero(), size,
            V2::zero(), padding)?, elements: Vec::new(), background,
            reactor: DefaultUIReactor::new()
        })
    }

    pub fn default(ctx: &mut Context, alignment: UIAlignment, pos: V2, size: V2,
        padding: f32) -> tetra::Result<Grid> {
        Ok(Grid {
            alignment, layout: UILayout::Default, transform: UITransform::new(ctx,
            V2::zero(), size, pos, padding)?, elements: Vec::new(), background: None,
            reactor: DefaultUIReactor::new()
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
        let element = self.elements.remove(index);
        self.update_element_alignments(index);
        element
    }

    pub fn clear_elements(&mut self) {
        self.elements.clear()
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

    fn get_pos(&self, ctx: &mut Context, parent_pos: V2) -> V2 {
        let size = self.transform.size;
        match self.layout {
            UILayout::Default => self.transform.get_abs_pos(parent_pos),
            UILayout::BottomLeft => V2::new(0.0, get_height(ctx) as f32 - size.y),
            UILayout::Centre => {
                let window_centre = V2::new(
                    get_width(ctx) as f32, get_height(ctx) as f32) * 0.5;
                window_centre - size * 0.5
            },
            UILayout::TopRight => V2::new(get_width(ctx) as f32 - size.x, 0.0),
            UILayout::BottomRight => V2::new(get_width(ctx) as f32,
                get_height(ctx) as f32) - size,
        }
    }
}

impl UIElement for Grid {
    fn get_name(&self) -> &str {
        "Grid"
    }

    fn get_reactor(&self) -> Option<&dyn UIReactor> {
        Some(&self.reactor)
    }

    fn get_reactor_mut(&mut self) -> Option<&mut dyn UIReactor> {
        Some(&mut self.reactor)
    }

    fn get_transform(&self) -> &UITransform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut UITransform {
        &mut self.transform
    }

    fn draw_rect(&mut self, ctx: &mut Context, parent_pos: V2) {
        // Not very useful most of the time
    }

    fn update_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        let pos = self.get_pos(ctx, parent_pos);
        for element in self.elements.iter() {
            let mut element_ref = element.borrow_mut();
            if !element_ref.is_active() {
                continue
            }

            element_ref.update_element(ctx, pos)?;
            element_ref.update_reactor(ctx, pos)?;
        }
        Ok(())
    }

    fn draw_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        let pos = self.get_pos(ctx, parent_pos);
        if let Some(background) = self.background.as_mut() {
            background.draw_element(ctx, pos)?;
        }
        for element in self.elements.iter() {
            let mut element_ref = element.borrow_mut();
            if !element_ref.is_invisible() {
                element_ref.draw_element(ctx, pos)?;
            }
            element_ref.draw_rect(ctx, pos);
        }
        Ok(())
    }

    fn event_element(&mut self, ctx: &mut Context, event: Event, parent_pos: V2)
        -> tetra::Result {
        let pos = self.get_pos(ctx, parent_pos);
        for element in self.elements.iter() {
            let mut element_ref = element.borrow_mut();
            if !element_ref.is_active() {
                continue
            }

            element_ref.event_element(ctx, event.clone(), pos)?;
        }
        Ok(())
    }
}
