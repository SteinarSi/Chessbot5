use crate::backend::board_representation::{movement, board};

pub trait AI{
	fn new() -> Self where Self: Sized;
	fn search(&mut self, b: board::Board) -> movement::Move;
	fn set_depth(&mut self, depth: usize);
	fn get_name(&self) -> &str;
}