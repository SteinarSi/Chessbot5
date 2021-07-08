use crate::backend::{movement, board};
use super::interface::AI;

use std::collections::HashMap;

const DEPTH: usize = 5;

pub struct MemoMax{
	memo: HashMap<i64, Memory>
}

struct Memory{
	value: movement::Score,
	depth: usize,
}

impl AI for MemoMax{
	fn new() -> Self{
		MemoMax{memo: HashMap::new()}
	} 
	fn search(&mut self, mut b: board::Board) -> movement::Move{
		let ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move when no moves are available"); }
		else if ms.len() == 1 { return ms[0]; }

		let mut ret = Vec::new();
		if b.color_to_move() == board::White{
			for mut m in ms.into_iter(){
				b.move_piece(&m);
				m.set_actual_value(self.mini(&mut b, DEPTH-1));
				b.go_back();
				ret.push(m);
			}
			ret.sort_by(|m1, m2| m2.actual_value().cmp(&m1.actual_value()));
		}
		else{
			for mut m in ms.into_iter(){
				b.move_piece(&m);
				m.set_actual_value(self.maxi(&mut b, DEPTH-1));
				b.go_back();
				ret.push(m);
			}
			ret.sort_by(|m1, m2| m1.actual_value().cmp(&m2.actual_value()));
		}

		return ret[0];
	}
}

impl MemoMax{
	fn maxi(&mut self, b: &mut board::Board, depth: usize) -> movement::Score {
		if depth == 0 { return b.heurestic_value(); }

		let mut ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		if let Some(m) = self.memo.get(&b.hash()){
			if m.depth >= depth { return m.value; }
		}

		let mut ret = - movement::INFINITY;
		for m in ms{
			b.move_piece(&m);
			ret = ret.max(self.mini(b, depth-1));
			b.go_back();
		}
		self.memo.insert(b.hash(), Memory{value: ret, depth});
		ret
	}

	fn mini(&mut self, b: &mut board::Board, depth: usize) -> movement::Score{
		if depth == 0 { return b.heurestic_value(); }

		let mut ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		if let Some(m) = self.memo.get(&b.hash()){
			if m.depth >= depth { return m.value; }
		}

		let mut ret = movement::INFINITY;
		for m in ms{
			b.move_piece(&m);
			ret = ret.min(self.maxi(b, depth-1));
			b.go_back();
		}
		self.memo.insert(b.hash(), Memory{value: ret, depth});
		ret
	}
}
