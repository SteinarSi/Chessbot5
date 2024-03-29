use crate::backend::board_representation::{movement::*, board::*};
use super::structures::{memomap::*, killerray::*};
use super::interface::AI;

const INITIAL_DEPTH: usize = 8;


// Alfa-beta-pruning, memoisering, i tillegg til å lagre alle trekk som forårsaker beta-cutoffs i en array.
// De trekkene kalles 'Killer Moves', og enhver posisjon vil først evaluere dybdens Killer Move om det er lovlig før de andre trekkene.
pub struct NegaAlphaKiller{
	memo: MemoMap,
	depth: usize,
	killerray: Killerray
}

impl AI for NegaAlphaKiller{
	fn new() -> Self{
		NegaAlphaKiller{memo: MemoMap::new(), depth: INITIAL_DEPTH, killerray: Killerray::new()}
	}

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn get_name(&self) -> &str {
		"NegaAlphaKiller"
	}

	fn search(&mut self, mut b: Board) -> Move{
		self.alphabeta(&mut b, - INFINITY, INFINITY, self.depth);
		self.memo.clean();
		self.memo.get(&b.hash()).unwrap().best.unwrap()
	}
}

impl NegaAlphaKiller{
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
	fn alphabeta(&mut self, b: &mut Board, mut alpha: Score, mut beta: Score, depth: usize) -> Score{
		if b.is_draw_by_repetition() { return 0; }
		if depth <= 0 { return b.symmetric_heuristic_value(); }

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
					let value = - self.alphabeta(b, -beta, -alpha, depth-1);
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
			if b.is_legal(&m){
				m.set_heuristic_value(b.value_of(&m));
				kill = Some(m);
				b.move_piece(&m);
				let value = - self.alphabeta(b, -beta, -alpha, depth-1);
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
		if ms.len() == 0 { return b.symmetric_end_score(); }

		ms.sort_by_heuristic(White);
		for m in ms{
			if Some(m) == prev || Some(m) == kill { continue; }
			b.move_piece(&m);
			let value = - self.alphabeta(b, -beta, -alpha, depth-1);
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
}