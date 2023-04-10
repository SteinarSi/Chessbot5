use crate::backend::board_representation::{movement, board};
use super::interface::AI;


// En ren minimax-algoritme. 
pub struct MiniMax{
	depth: usize
}

const INITIAL_DEPTH: usize = 5;

impl AI for MiniMax{
	fn new() -> Self{
		MiniMax{depth: INITIAL_DEPTH}
	}

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn search(&mut self, mut b: board::Board) -> board::Move{
		let ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move when no legal moves are available"); }
		let mut ret = Vec::new();

		if b.color_to_move() == board::Color::White{
			for m in ms.into_iter(){
				b.move_piece(&m);
				ret.push((m, self.mini(&mut b, self.depth-1)));
				b.go_back();
			}
			ret.sort_by(|m1, m2| m2.1.cmp(&m1.1));
		}
		else{
			for m in ms.into_iter(){
				b.move_piece(&m);
				ret.push((m, self.maxi(&mut b, self.depth-1)));
				b.go_back();
			}
			ret.sort_by(|m1, m2| m1.1.cmp(&m2.1));
		}
		ret[0].0
	}

	fn get_name(&self) -> &str {
		"MiniMax"
	}
}

impl MiniMax{
	fn maxi(&self, b: &mut board::Board, depth: usize) -> movement::Score{
		if b.is_draw_by_repetition() { 0 }
		else if depth == 0 { b.heuristic_value() }
		else{
			let ms = b.moves();
			if ms.len() == 0 { return b.end_score(); }
			let mut ret = - movement::INFINITY;
			for m in ms{
				b.move_piece(&m);
				ret = ret.max(self.mini(b, depth-1));
				b.go_back();
			}
			ret
		}
	}

	fn mini(&self, b: &mut board::Board, depth: usize) -> movement::Score{
		if b.is_draw_by_repetition() { 0 }
		else if depth == 0 { b.heuristic_value() }
		else{
			let ms = b.moves();
			if ms.len() == 0 { return b.end_score(); }
			let mut ret  = movement::INFINITY;
			for m in ms{
				b.move_piece(&m);
				ret = ret.min(self.maxi(b, depth-1));
				b.go_back();
			}
			ret
		}
	}
}
