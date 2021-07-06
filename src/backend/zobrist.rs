use crate::backend::piece::{Piece, Color, Color::*, PieceType::*};
use crate::backend::movement::Position;

struct Zobrist{
	hash: i64,

}

impl Zobrist{
	//pub fn new() -> Self{
		//TODO
	//}
	pub fn update(&mut self, p: &Piece, pos: &Position){

	}
}


#[cfg(test)]
mod zobrist_testing{

}
