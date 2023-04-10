use crate::backend::board_representation::{movement::*, board::*};
use super::interface::AI;

const INITIAL_DEPTH: usize = 6;

// NegaMax med alfa-beta pruning, uten memoisering
pub struct NegaAlpha{
	depth: usize
}

impl AI for NegaAlpha{
	fn new() -> Self{
		NegaAlpha{depth: INITIAL_DEPTH}
	}

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn get_name(&self) -> &str {
		"NegaAlpha"
	}

	fn search(&mut self, mut b: Board) -> Move{
		let mut ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move with no moves to choose from!"); }
		ms.sort_by_heuristic(b.color_to_move());
		
		let mut alpha = - INFINITY;
		let mut best = ms[0];
		for m in ms{
			b.move_piece(&m);
			let value = - self.alphabeta(&mut b, - INFINITY, - alpha, self.depth-1);
			//print!("{} ", value);
			b.go_back();
			if value > alpha{
				alpha = value;
				best = m;
			}
		}
		//println!("\nbest: {:?} at {}.", best, bestScore);
		best
	}
}

impl NegaAlpha{
	fn alphabeta(&mut self, b: &mut Board, mut alpha: Score, beta: Score, depth: usize) -> Score{
		if b.is_draw_by_repetition() { return 0; }
		if depth <= 0 { return b.symmetric_heuristic_value(); }
		let mut ms = b.moves();
		if ms.len() == 0 { return b.symmetric_end_score(); }

		ms.sort_by_heuristic(b.color_to_move());

		let mut value = - INFINITY;
		for m in ms{
			b.move_piece(&m);
			value = value.max( - self.alphabeta(b, -beta, -alpha, depth-1));
			b.go_back();
			alpha = alpha.max(value);
			if value >= beta{
				break; //beta cutoff
			}
		}
		value
	}
}