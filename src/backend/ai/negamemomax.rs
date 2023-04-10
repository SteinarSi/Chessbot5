use crate::backend::board_representation::{movement::*, board::*};
use super::interface::AI;

use std::collections::HashMap;

const INITIAL_DEPTH: usize = 5;

// MiniMax, men med memoisering.
// Har den sett og evaluert en posisjon før, bruker den det resultatet.
pub struct NegaMemoMax{
	memo: HashMap<i64, Memory>,
	depth: usize
}

struct Memory{
	value: Score,
	depth: usize,
}

impl AI for NegaMemoMax{
	fn new() -> Self{
		NegaMemoMax{memo: HashMap::new(), depth: INITIAL_DEPTH}
	} 

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn get_name(&self) -> &str {
		"NegaMemoMax"
	}

	fn search(&mut self, mut b: Board) -> Move{
		let ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move when no moves are available"); }

		let mut best_move = ms[0];
		let mut best_value = - INFINITY;
        for m in ms.into_iter(){
            b.move_piece(&m);
			let value = - self.memomax(&mut b, self.depth-1);
            b.go_back();
			if value > best_value {
				best_value = value;
				best_move = m;
			}
        }
        best_move
	}
}

impl NegaMemoMax{
	fn memomax(&mut self, b: &mut Board, depth: usize) -> Score {
		if b.is_draw_by_repetition() { return 0; }
		if depth == 0 { return b.symmetric_heuristic_value(); }

		if let Some(m) = self.memo.get(&b.hash()){
			if m.depth >= depth { return m.value; }
		}

		let ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		let mut ret = - INFINITY;
		for m in ms{
			b.move_piece(&m);
			ret = ret.max( - self.memomax(b, depth-1));
			b.go_back();
		}
		self.memo.insert(b.hash(), Memory{value: ret, depth});
		ret
	}
}