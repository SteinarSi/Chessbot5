use crate::backend::board_representation::{movement::*, board::*};
use super::structures::{memomap::*, killerray::*};
use super::interface::AI;

const INITIAL_DEPTH: usize = 8;

// Alfa-beta-pruning, memoisering, killer heurestic, samt et quiescence search når depth=1.
// 'Quiesce' betyr 'stille' på fransk, og betyr i vår kontekst å kun evaluere 'rolige' trekk.
// Vi definerer et 'rolig' trekk som et trekk som flytter til en rute som ikke blir truet av motstanderen.
pub struct NegaQuiescence{
	memo: MemoMap,
	depth: usize,
	killerray: Killerray
}

impl AI for NegaQuiescence{
	fn new() -> Self{
		NegaQuiescence{memo: MemoMap::new(), depth: INITIAL_DEPTH, killerray: Killerray::new()}
	}

	fn set_depth(&mut self, depth: usize){
        if depth < 2 { panic!("Depth has to be at least 2"); }
		self.depth = depth;
	}

	fn get_name(&self) -> &str {
		"NegaQuiescence"
	}

	fn search(&mut self, mut b: Board) -> Move{
		self.alphabeta(&mut b, -INFINITY, INFINITY, self.depth);
		self.memo.clean();
		self.memo.get(&b.hash()).unwrap().best.unwrap()
	}
}

impl NegaQuiescence{
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

	fn quiesce(&mut self, b: &mut Board, beta: Score) -> Score{
		//Om den nåværende scoren allerede er uakseptabel for svart kan vi anta at hvits neste trekk gjør den enda mer uakseptabel.
		//Da er det ikke vits i å sjekke engang. Dette antar altså et hvit ikke er i zugswang.
		let stand_pat = b.symmetric_heuristic_value();
		if stand_pat >= beta { return stand_pat; } 

		let ms = b.moves();
		if ms.len() == 0 { return b.symmetric_end_score(); }

		//Filtrerer trekk, vi vil kun ha trekk som flytter til en rute som ikke er truet.
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
			ms.sort_by_heuristic(b.color_to_move());
			(b.heuristic_value() + ms[0].heuristic_value()) * if b.color_to_move() == White { 1 } else { -1 }
		}
	}

	fn alphabeta(&mut self, b: &mut Board, mut alpha: Score, mut beta: Score, depth: usize) -> Score{
		if b.is_draw_by_repetition() { return 0; }
		if depth <= 1 { return self.quiesce(b, beta); }

		let mut best = None;
		let mut exact = false;
		let mut prev = None;
		let mut kill = None;

		//Sjekker om vi allerede har et relevant oppslag i hashmappet vårt.
		if let Some(t) = self.memo.get(&b.hash()){
			if t.depth >= depth{
				//Hvis dybden på oppslaget er nok, kan vi bruke verdiene umiddelbart.
				match &t.flag{
					TransFlag::EXACT       => { return t.value; }
					TransFlag::LOWER_BOUND => { alpha = alpha.max(t.value); }
					TransFlag::UPPER_BOUND => { beta  = beta .min(t.value); }
				}
				if alpha >= beta { return t.value; }
			}
			//Hvis ikke kan vi ikke bruke verdiene.
			//Da sjekker vi istedet om oppslaget har lagret et bra trekk, og evaluerer i så fall det trekket først.
			else if let Some(m) = t.best{
				if b.is_legal(&m) { 
					prev = t.best;
					b.move_piece(&m);
					let value = - self.alphabeta(b, -beta, -alpha, depth-1);
					b.go_back();

					//Beta-cutoff, stillingen er uakseptabel for svart.
					if value >= beta{
						self.killerray.put(m.clone(), b.counter());
						self.memo.insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
						return value;
					}

					//Trekket forbedret alfa, dette er dermed en PV-node.
					if value > alpha{
						best = Some(m);
						exact = true;
						alpha = value;
					}
				}
			}
		}

		//Sjekker om vi har et 'Killer Move' for denne dybden.
		//Da evaluerer vi det trekker først.
		if let Some(mut m) = self.killerray.get(b.counter()){
			if b.is_legal(&m) && Some(m) != prev{
				m.set_heuristic_value(b.value_of(&m));
				kill = Some(m);
				b.move_piece(&m);
				let value = - self.alphabeta(b, -beta, -alpha, depth-1);
				b.go_back();

				//Beta-cutoff, stillingen er uakseptabel for svart.
				if value >= beta{
					self.memo.insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
					return value;
				}

				//Trekket forbedret alfa.
				if value > alpha{
					best = Some(m);
					exact = true;
					alpha = value;
				}
			}
		}

		// Først her, når vi allerede har vurdert eventuelle 'Killer'-trekk og memoiserte trekk,
		// begynner vi å generere listen av alle trekk og evaluerer dem.
		let mut ms = b.moves();
		if ms.len() == 0 { return b.symmetric_end_score(); }

		ms.sort_by_heuristic(b.color_to_move());
		for m in ms{
			if Some(m) == prev || Some(m) == kill { continue; }
			b.move_piece(&m);
			let value = - self.alphabeta(b, -beta, -alpha, depth-1);
			b.go_back();

			//Beta-cutoff
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