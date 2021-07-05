use crate::backend::{movement, board};

pub trait AI{
	fn new() -> Self;
	fn search(&mut self, b: board::Board) -> movement::Move;
}