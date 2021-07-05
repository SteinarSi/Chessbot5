use crate::backend::{movement, board};
use super::interface::AI;

pub struct MiniMax{
	//No fields, this is a stupid AI
}

const DEPTH: i8 = 4;
const INFINITY: movement::Score = 2147483647;

impl AI for MiniMax{
	fn search(&mut self, mut b: board::Board) -> board::Move{
		let ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move when no legal moves are available"); }
		let mut ret = Vec::new();

		if b.color_to_move() == board::Color::White{
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

		for m in &ret{
			println!("{}: {}", m.to_string(), m.actual_value());
		}
		ret[0]
	}

	fn new() -> Self{
		MiniMax{}
	}
}

impl MiniMax{
	fn maxi(&self, b: &mut board::Board, depth: i8) -> movement::Score{
		if depth == 0 { b.heurestic_value() }
		else{
			let ms = b.moves();
			if ms.len() == 0 { return - INFINITY + b.counter() as i32; }
			let mut ret = - INFINITY;
			for m in ms{
				b.move_piece(&m);
				ret = ret.max(self.mini(b, depth-1));
				b.go_back();
			}
			ret
		}
	}

	fn mini(&self, b: &mut board::Board, depth: i8) -> movement::Score{
		if depth == 0 { b.heurestic_value() }
		else{
			let ms = b.moves();
			if ms.len() == 0 { return INFINITY - b.counter() as i32; }
			let mut ret  = INFINITY;
			for m in ms{
				b.move_piece(&m);
				ret = ret.min(self.maxi(b, depth-1));
				b.go_back();
			}
			ret
		}
	}
}
