use crate::backend::{movement::*, board::*};
use super::structures::memomap::*;
use super::interface::AI;

const INITIAL_DEPTH: usize = 8;


// Alfa-beta-pruning, med memoisering.
// Har den sett og evaluert en posisjon før vil den sammenligne den med alfa og beta.
// Hvis posisjonen må evalueres lengre ser den om den har et lagret beste trekk, og evaluerer det først.
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

	fn search(&mut self, mut b: Board) -> Move{
		if b.color_to_move() == White{
			self.maximize_alpha(&mut b, - INFINITY, INFINITY, self.depth);
		}
		else{
			self.minimize_beta(&mut b, - INFINITY, INFINITY, self.depth);
		}
		self.memo.clean();
		self.memo.get(&b.hash()).unwrap().best.unwrap()
	}
}

impl MemoAlpha{
	pub fn principal_variation(&mut self, mut b: Board) -> Moves{
		let mut ret = Moves::new();
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
	fn maximize_alpha(&mut self, b: &mut Board, mut alpha: Score, mut beta: Score, depth: usize) -> Score{
		if depth <= 0 { return b.heuristic_value(); }

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
				b.move_piece(&m);
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

		ms.sort_by_heuristic(White);

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

	fn minimize_beta(&mut self, b: &mut Board, mut alpha: Score, mut beta: Score, depth: usize) -> Score{
		if depth <= 0 { return b.heuristic_value(); }

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
		ms.sort_by_heuristic(Black);

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