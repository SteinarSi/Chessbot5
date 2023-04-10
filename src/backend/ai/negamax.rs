use crate::backend::board_representation::{movement::{Score, INFINITY}, board::{Board, Move, Color::White}};
use super::interface::AI;


// En ren minimax-algoritme. 
pub struct NegaMax{
	depth: usize
}

const INITIAL_DEPTH: usize = 5;

impl AI for NegaMax{
	fn new() -> Self{
		NegaMax{depth: INITIAL_DEPTH}
	}

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn search(&mut self, mut b: Board) -> Move{
		let ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move when no legal moves are available"); }
		let mut best_value = -INFINITY;
		let mut best_move = ms[0];

		for m in ms.into_iter(){
			b.move_piece(&m);
			let value = - self.negamax(&mut b, self.depth-1);
			b.go_back();	
			if value > best_value {
				best_value = value;
				best_move = m;
			}
		}
		best_move
	}

	fn get_name(&self) -> &str {
		"NegaMax"
	}
}

impl NegaMax{
	fn negamax(&self, b: &mut Board, depth: usize) -> Score{
		if depth == 0 { return b.symmetric_heuristic_value(); }
		if b.is_draw_by_repetition() { return 0; }
		let ms = b.moves();
		if ms.len() == 0 { return b.symmetric_end_score(); }

		let mut ret = - INFINITY;
		for m in ms {
			b.move_piece(&m);
			ret = ret.max( - self.negamax(b, depth-1));
			b.go_back();
		};
		ret
	}
}