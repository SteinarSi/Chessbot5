use crate::backend::{movement::*, board::*};
use super::structures::{memomap::*, killerray::*};
use super::interface::AI;

const INITIAL_DEPTH: usize = 8;

pub struct Quiescence{
	memo: MemoMap,
	depth: usize,
	killerray: Killerray
}

impl AI for Quiescence{
	fn new() -> Self{
		Quiescence{memo: MemoMap::new(), depth: INITIAL_DEPTH, killerray: Killerray::new()}
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

impl Quiescence{
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

	fn quimax(&mut self, b: &mut Board, beta: Score) -> Score{
		let stand_pat = b.heuristic_value();
		if stand_pat >= beta { return stand_pat; }

		let ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		let mut arr = [[None; 8]; 8];
		let filt = |m: &Move| {
			if let Some(bo) = arr[m.to.y][m.to.x]{
				bo
			}else{
				let bo = ! b.is_threatened(&m.to);
				arr[m.to.y][m.to.x] = Some(bo);
				bo
			}
		};

		let mut ms: Moves = ms.into_iter().filter(filt).collect();
		if ms.len() == 0 { stand_pat }
		else {
			ms.sort_by_heuristic(White);
			b.heuristic_value() + ms[0].heuristic_value()
		}
	}


	fn quimin(&mut self, b: &mut Board, alpha: Score) -> Score{
		let stand_pat = b.heuristic_value();
		if stand_pat <= alpha { return stand_pat; }

		let ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		let mut arr = [[None; 8]; 8];
		let filt = |m: &Move| {
			if let Some(bo) = arr[m.to.y][m.to.x]{
				bo
			}else{
				let bo = ! b.is_threatened(&m.to);
				arr[m.to.y][m.to.x] = Some(bo);
				bo
			}
		};

		let mut ms: Moves = ms.into_iter().filter(filt).collect();
		if ms.len() == 0 { stand_pat }
		else {
			ms.sort_by_heuristic(Black);
			b.heuristic_value() + ms[0].heuristic_value()
		}

	}

	fn maximize_alpha(&mut self, b: &mut Board, mut alpha: Score, mut beta: Score, depth: usize) -> Score{
		if depth <= 1 { return self.quimax(b, beta); }

		let mut best = None;
		let mut exact = false;
		let mut prev = None;
		let mut kill = None;

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
				if b.is_legal(&m) { 
					prev = t.best;
					b.move_piece(&m);
					let value = self.minimize_beta(b, alpha, beta, depth-1);
					b.go_back();

					if value >= beta{
						self.killerray.put(m.clone(), b.counter());
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
		}
		if let Some(mut m) = self.killerray.get(b.counter()){
			if b.is_legal(&m) && Some(m) != prev{
				m.set_heuristic_value(b.value_of(&m));
				kill = Some(m);
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
			if Some(m) == prev || Some(m) == kill { continue; }
			b.move_piece(&m);
			let value = self.minimize_beta(b, alpha, beta, depth-1);
			b.go_back();

			if value >= beta{
				self.killerray.put(m.clone(), b.counter());
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
		if depth <= 1 { return self.quimin(b, alpha); }

		let mut exact = false;
		let mut best = None;
		let mut prev = None;
		let mut kill = None;

		if let Some(t) = self.memo.get(&b.hash()){
			if t.depth >= depth{
				match &t.flag{
					TransFlag::EXACT       => { return t.value; }
					TransFlag::LOWER_BOUND => { alpha = alpha.max(t.value); }
					TransFlag::UPPER_BOUND => { beta  = beta .min(t.value); }
				}
			}
			else if let Some(m) = t.best{
				if b.is_legal(&m){
					prev = t.best;
					b.move_piece(&m);
					let value = self.maximize_alpha(b, alpha, beta, depth-1);
					b.go_back();

					if value <= alpha{
						self.killerray.put(m, b.counter());
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
		}

		if let Some(mut m) = self.killerray.get(b.counter()){
			if b.is_legal(&m) && Some(m) != prev{
				m.set_heuristic_value(b.value_of(&m));
				kill = Some(m);
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
			if Some(m) == prev || Some(m) == kill { continue; }
			b.move_piece(&m);
			let value = self.maximize_alpha(b, alpha, beta, depth-1);
			b.go_back();

			if value <= alpha{
				self.killerray.put(m, depth);
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