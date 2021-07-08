use crate::backend::{movement, board};
use super::interface::AI;

use std::collections::HashMap;

const DEPTH: usize = 5;

pub struct MemoAlpha{
	memo: HashMap<i64, Memory>
}

struct Memory{
	value: movement::Score,
	depth: usize,
	flag: TransFlag
}

enum TransFlag{
	EXACT,
	UPPER_BOUND,
	LOWER_BOUND
}

impl AI for MemoAlpha{
	fn new() -> Self{
		MemoAlpha{memo: HashMap::new()}
	}

	fn search(&mut self, mut b: board::Board) -> movement::Move{
		let mut ms = b.moves();
		if ms.len() == 0 { panic!("Cannot pick a move when none are available"); }
		ms[0] //TODOOOOO
	}
}