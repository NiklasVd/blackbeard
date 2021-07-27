use tetra::{Context, State};
use crate::GC;

pub trait Entity<T> : State {
    fn init(ctx: &mut Context, game: GC) -> tetra::Result<T>;
}
