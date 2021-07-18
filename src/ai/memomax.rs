use crate::backend::{movement::*, board::*};
use super::interface::AI;

use std::collections::HashMap;

const INITIAL_DEPTH: usize = 5;

// MiniMax, men med memoisering.
// Har den sett og evaluert en posisjon før, bruker den det resultatet.
pub struct MemoMax{
	memo: HashMap<i64, Memory>,
	depth: usize
}

struct Memory{
	value: Score,
	depth: usize,
}

impl AI for MemoMax{
	fn new() -> Self{
		MemoMax{memo: HashMap::new(), depth: INITIAL_DEPTH}
	} 

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn search(&mut self, mut b: Board) -> Move{
		let ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move when no moves are available"); }

		let mut ret = Moves::new();
		if b.color_to_move() == White{
			for mut m in ms.into_iter(){
				b.move_piece(&m);
				m.set_actual_value(self.mini(&mut b, self.depth-1));
				b.go_back();
				ret.push(m);
			}
			ret.sort_by_actual(White);
		}
		else{
			for mut m in ms.into_iter(){
				b.move_piece(&m);
				m.set_actual_value(self.maxi(&mut b, self.depth-1));
				b.go_back();
				ret.push(m);
			}
			ret.sort_by_actual(Black);
		}

		return ret[0];
	}
}

impl MemoMax{
	fn maxi(&mut self, b: &mut Board, depth: usize) -> Score {
		if depth == 0 { return b.heuristic_value(); }

		if let Some(m) = self.memo.get(&b.hash()){
			if m.depth >= depth { return m.value; }
		}

		let ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		let mut ret = - INFINITY;
		for m in ms{
			b.move_piece(&m);
			ret = ret.max(self.mini(b, depth-1));
			b.go_back();
		}
		self.memo.insert(b.hash(), Memory{value: ret, depth});
		ret
	}

	fn mini(&mut self, b: &mut Board, depth: usize) -> Score{
		if depth == 0 { return b.heuristic_value(); }

		if let Some(m) = self.memo.get(&b.hash()){
			if m.depth >= depth { return m.value; }
		}

		let ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		let mut ret = INFINITY;
		for m in ms{
			b.move_piece(&m);
			ret = ret.min(self.maxi(b, depth-1));
			b.go_back();
		}
		self.memo.insert(b.hash(), Memory{value: ret, depth});
		ret
	}
}
