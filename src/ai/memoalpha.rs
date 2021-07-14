use crate::backend::{movement::*, board};
use super::structures::memomap::*;
use super::interface::AI;

const INITIAL_DEPTH: usize = 7;

pub struct MemoAlpha{
	memo: MemoMap,
	depth: usize
}

impl AI for MemoAlpha{
	fn new() -> Self{
		MemoAlpha{memo: MemoMap::new(), depth: INITIAL_DEPTH}
	}

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn search(&mut self, mut b: board::Board) -> Move{
		let ms = b.moves();
		println!("Options: {:?}", ms);
		if ms.len() == 0 { panic!("Cannot pick a move when none are available"); }
		if b.color_to_move() == board::White{
			self.maximize_alpha(&mut b, - INFINITY, INFINITY, self.depth);
		}
		else{
			self.minimize_beta(&mut b, - INFINITY, INFINITY, self.depth);
		}
		let deleted = self.memo.clean();
		println!("Move: {}\nMap size: {}\nDeleted entries: {}", self.memo.get(&b.hash()).unwrap().best.unwrap().to_string(), self.memo.len(), deleted);
		self.memo.get(&b.hash()).unwrap().best.unwrap()
	}
}

impl MemoAlpha{
	pub fn principal_variation(&mut self, mut b: board::Board) -> Vec<Move>{
		let mut ret = Vec::new();
		self.search(b.clone());
		loop{
			match self.memo.get(&b.hash()){
				None => { return ret; }
				Some(m) => {
					match m.best{
						None => { return ret; }
						Some(m) => {
							b.move_piece(&m);
							ret.push(m);
						}
					}
				}
			}
		}

	}
	fn maximize_alpha(&mut self, b: &mut board::Board, mut alpha: Score, mut beta: Score, depth: usize) -> Score{
		if depth <= 0 { return b.heurestic_value(); }

		let mut best = None;
		let mut exact = false;
		let mut prev = None;

		if let Some(t) = self.memo.get(&b.hash()){
			if t.depth >= depth{
				match &t.flag{
					TransFlag::EXACT       => { return t.value; }
					TransFlag::LOWER_BOUND => { alpha = alpha.max(t.value); }
					TransFlag::UPPER_BOUND => { beta  = beta .min(t.value); }
				}
				if alpha >= beta { return t.value; }
			}
			else if let Some(m) = t.best{
				prev = t.best;
				b.move_piece(&m); //TODO: en sjekk for om trekket er lovlig eller ei
				let value = self.minimize_beta(b, alpha, beta, depth-1);
				b.go_back();

				if value >= beta{
					self.memo.insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
					return value;
				}

				if value > alpha{
					best = Some(m);
					exact = true;
					alpha = value;
				}
			}
		}

		let mut ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		ms.sort_by(|m1, m2| m2.heurestic_value().cmp(&m1.heurestic_value()));

		for m in ms{
			if Some(m) == prev { continue; }
			b.move_piece(&m);
			let value = self.minimize_beta(b, alpha, beta, depth-1);
			b.go_back();

			if value >= beta{
				self.memo.insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
				return value;
			}

			if value > alpha{
				alpha = value;
				exact = true;
				best = Some(m);
			}
		}
		self.memo.insert(b.hash(), alpha, if exact {TransFlag::EXACT} else { TransFlag::UPPER_BOUND }, depth, best);
		alpha
	}

	fn minimize_beta(&mut self, b: &mut board::Board, mut alpha: Score, mut beta: Score, depth: usize) -> Score{
		if depth <= 0 { return b.heurestic_value(); }

		let mut exact = false;
		let mut best = None;
		let mut prev = None;

		if let Some(t) = self.memo.get(&b.hash()){
			if t.depth >= depth{
				match &t.flag{
					TransFlag::EXACT       => { return t.value; }
					TransFlag::LOWER_BOUND => { alpha = alpha.max(t.value); }
					TransFlag::UPPER_BOUND => { beta  = beta .min(t.value); }
				}
			}
			else if let Some(m) = t.best{
				prev = t.best;
				b.move_piece(&m);
				let value = self.maximize_alpha(b, alpha, beta, depth-1);
				b.go_back();

				if value <= alpha{
					self.memo.insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
					return value;
				}

				if value < beta{
					exact = true;
					beta = value;
					best = Some(m);
				}
			}
		}

		let mut ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }
		ms.sort_by(|m1, m2| m1.heurestic_value().cmp(&m2.heurestic_value()));

		for m in ms{
			if Some(m) == prev { continue; }
			b.move_piece(&m);
			let value = self.maximize_alpha(b, alpha, beta, depth-1);
			b.go_back();

			if value <= alpha{
				self.memo.insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
				return value;
			}

			if value < beta{
				exact = true;
				beta = value;
				best = Some(m);
			}
		}
		self.memo.insert(b.hash(), beta, if exact { TransFlag::EXACT } else { TransFlag::LOWER_BOUND }, depth, best);
		beta
	}
}