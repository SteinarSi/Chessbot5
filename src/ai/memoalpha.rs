use crate::backend::{movement::*, board};
use super::interface::AI;

use std::collections::HashMap;

const INITIAL_DEPTH: usize = 5;

pub struct MemoAlpha{
	memo: HashMap<i64, Memory>,
	depth: usize
}

struct Memory{
	value: Score,
	depth: usize,
	flag: TransFlag,
	best: Option<Move>
}

enum TransFlag{
	EXACT,
	UPPER_BOUND,
	LOWER_BOUND
}

impl AI for MemoAlpha{
	fn new() -> Self{
		MemoAlpha{memo: HashMap::new(), depth: INITIAL_DEPTH}
	}

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn search(&mut self, mut b: board::Board) -> Move{
		let mut ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move when none are available"); }
		if b.color_to_move() == board::White{
			self.maximize_alpha(&mut b, - INFINITY, INFINITY, self.depth);
		}
		else{
			self.minimize_beta(&mut b, - INFINITY, INFINITY, self.depth);
		}
		self.memo.get(&b.hash()).unwrap().best.unwrap()
	}
}

impl MemoAlpha{

	fn maximize_alpha(&mut self, b: &mut board::Board, mut alpha: Score, beta: Score, depth: usize) -> Score{
		if depth <= 0 { return b.heurestic_value(); }

		if let Some(m) = self.memo.get(&b.hash()){
			if m.depth >= depth{
				match &m.flag{
					TransFlag::EXACT       => { return m.value; }
					TransFlag::LOWER_BOUND => { if m.value >= beta  { return m.value; } }
					TransFlag::UPPER_BOUND => { if m.value <= alpha { return m.value; } }
				}
			}
		}

		let mut ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		ms.sort_by(|m1, m2| m2.heurestic_value().cmp(&m1.heurestic_value()));

		let mut best = None;
		let mut exact = false;

		for m in ms{
			b.move_piece(&m);
			let value = self.minimize_beta(b, alpha, beta, depth-1);
			b.go_back();

			if value >= beta{
				self.memo.insert(b.hash(), Memory{value, depth, best: Some(m), flag: TransFlag::LOWER_BOUND});
				return value;
			}

			if value > alpha{
				alpha = value;
				exact = true;
				best = Some(m);
			}
		}
		self.memo.insert(b.hash(), Memory{value: alpha, depth, best, flag: if exact {TransFlag::EXACT} else { TransFlag::UPPER_BOUND }});
		alpha
	}

	fn minimize_beta(&mut self, b: &mut board::Board, alpha: Score, mut beta: Score, depth: usize) -> Score{
		if depth <= 0 { return b.heurestic_value(); }

		if let Some(m) = self.memo.get(&b.hash()){
			if m.depth >= depth{
				match &m.flag{
					TransFlag::EXACT       => { return m.value; }
					TransFlag::LOWER_BOUND => { if m.value >= beta  { return m.value; } }
					TransFlag::UPPER_BOUND => { if m.value <= alpha { return m.value; } }
				}
			}
		}

		let mut ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }
		ms.sort_by(|m1, m2| m1.heurestic_value().cmp(&m2.heurestic_value()));

		let mut exact = false;
		let mut best = None;

		for m in ms{
			b.move_piece(&m);
			let value = self.maximize_alpha(b, alpha, beta, depth-1);
			b.go_back();

			if value <= alpha{
				self.memo.insert(b.hash(), Memory{value, depth, best: Some(m), flag: TransFlag::UPPER_BOUND});
				return value;
			}

			if value < beta{
				exact = true;
				beta = value;
				best = Some(m);
			}
		}
		self.memo.insert(b.hash(), Memory{value: beta, depth, best, flag: if exact { TransFlag::EXACT } else { TransFlag::LOWER_BOUND }});
		beta
	}
}