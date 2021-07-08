use crate::backend::{movement, board};
use super::interface::AI;

const DEPTH: usize = 5;

pub struct AlphaBeta{
	
}

impl AI for AlphaBeta{
	fn new() -> Self{
		AlphaBeta{}
	}

	fn search(&mut self, mut b: board::Board) -> movement::Move{
		let mut ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move with no moves to choose from!"); }

		let mut ret = Vec::new();
		if b.color_to_move() == board::White{
			ms.sort_by(|m1, m2| m2.heurestic_value().cmp(&m1.heurestic_value()));
			let mut alpha = - movement::INFINITY;
			for mut m in ms{
				b.move_piece(&m);
				let value = self.minimize_beta(&mut b, alpha, movement::INFINITY, DEPTH-1);
				alpha = alpha.max(value);
				m.set_actual_value(value);
				b.go_back();
				ret.push(m);
			}
			ret.sort_by(|m1, m2| m2.actual_value().cmp(&m1.actual_value()));
			ret[0]
		}
		else{
			ms.sort_by(|m1, m2| m1.heurestic_value().cmp(&m2.heurestic_value()));
			let mut beta = movement::INFINITY;
			for mut m in ms{
				b.move_piece(&m);
				let value = self.maximize_alpha(&mut b, - movement::INFINITY, beta, DEPTH-1);
				beta = beta.min(value);
				m.set_actual_value(value);
				b.go_back();
				ret.push(m);
			}
			ret.sort_by(|m1, m2| m1.actual_value().cmp(&m2.actual_value()));
			ret[0]
		}
	}
}

impl AlphaBeta{
	fn maximize_alpha(&mut self, b: &mut board::Board, mut alpha: movement::Score, beta: movement::Score, depth: usize) -> movement::Score{
		if depth <= 0 { return b.heurestic_value(); }
		let mut ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		ms.sort_by(|m1, m2| m2.heurestic_value().cmp(&m1.heurestic_value()));

		let mut value = - movement::INFINITY;
		for m in ms{
			b.move_piece(&m);
			value = value.max(self.minimize_beta(b, alpha, beta, depth-1));
			b.go_back();
			alpha = alpha.max(value);
			if value >= beta{
				break; //beta cutoff
			}
		}
		value
	}

	fn minimize_beta(&mut self,  b: &mut board::Board, alpha: movement::Score, mut beta: movement::Score, depth: usize) -> movement::Score{
		if depth <= 0 { return b.heurestic_value(); }
		let mut ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

		ms.sort_by(|m1, m2| m1.heurestic_value().cmp(&m2.heurestic_value()));

		let mut value = movement::INFINITY;
		for m in ms{
			b.move_piece(&m);
			value = value.min(self.maximize_alpha(b, alpha, beta, depth-1));
			b.go_back();
			beta = beta.min(value);
			if value <= alpha{
				break; //alpha cutoff
			}
		}
		value
	}
}