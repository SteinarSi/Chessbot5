pub mod minimax;

use crate::backend::{movement, board};

trait AI{
	fn search(&mut self, b: board::Board) -> movement::Move;
}