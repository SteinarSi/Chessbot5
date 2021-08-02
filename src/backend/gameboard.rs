use crate::backend::board_representation::board::*;
use crate::backend::board_representation::movement::*;
use crate::backend::ai::interface::AI;
use crate::backend::ai::omikron::Omikron;

pub struct Gameboard{
	board: Board,
	ai: Box<AI>
}

impl Gameboard{
	pub fn new() -> Self{
		Gameboard{ai: Box::new(Omikron::new()), board: Board::new()}
	}

	pub fn move_piece(&mut self, s: &str) -> Result<(), std::io::Error>{
		self.board.move_str(s)
	}

	pub fn moves(&mut self) -> Moves{
		self.board.moves()
	}

	pub fn moves_from(&mut self, x: usize, y: usize) -> Moves{
		self.moves().into_iter().filter(|m| m.from == Position{x, y}).collect()
	}

	pub fn start_bot(&mut self){
		let m = self.ai.search(self.board.clone());
		self.board.move_piece(&m);
	}

	pub fn is_checkmate(&mut self) -> bool{
		self.board.is_checkmate()
	}

	pub fn winner(&mut self) -> Option<Color>{
		if self.board.is_checkmate() || self.board.is_draw_by_repetition(){
			self.board.winner()
		}else{
			panic!("Why would you ask for a winner when the game is still ongoing!");
		}
	}
}