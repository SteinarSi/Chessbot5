use std::collections::HashMap;
use crate::backend::board::{Move, Score};

pub type Key = i64;
const LIFETIME: u8 = 3;
const ADD: u8 = LIFETIME - 1;

pub struct MemoMap{
	map: HashMap<Key, Transposition>,
	delete: u8
}

#[derive(PartialEq, Debug)]
pub struct Transposition{
	pub value: Score,
	pub flag: TransFlag,
	pub depth: usize,
	pub best: Option<Move>,
	age: u8
}

#[derive(PartialEq, Debug)]
pub enum TransFlag{
	EXACT,
	UPPER_BOUND,
	LOWER_BOUND
}

impl MemoMap{
	pub fn new() -> Self{
		MemoMap{map: HashMap::with_capacity(7_500_000), delete: 0}
	}

	pub fn get(&self, k: &Key) -> Option<&Transposition>{
		self.map.get(k)
	}

	pub fn insert(&mut self, k: Key, value: Score, flag: TransFlag, depth: usize, best: Option<Move>){
		self.map.insert(k, Transposition{value, flag, depth, best, age: (self.delete + ADD) % LIFETIME});
	}

	pub fn clean(&mut self) -> usize{
		self.delete = (self.delete + 1) % LIFETIME;
		let before = self.map.len();
		let delete = self.delete;
		self.map.retain(|_, v| (*v).age != delete);

		before - self.map.len()
	}

	pub fn len(&self) -> usize{
		self.map.len()
	}
}

#[cfg(test)]
mod map_tests{
	use super::*;

	#[test]
	fn can_insert_and_retrieve(){
		let mut memo = MemoMap::new();

		memo.insert(13, 0, TransFlag::EXACT, 0, None);

		assert_eq!(Some(&Transposition{value: 0, flag: TransFlag::EXACT, depth: 0, best: None, age: 2}), memo.get(&13));
	}

	#[test]
	fn cleaning_twice_removes_entry(){
		let mut memo = MemoMap::new();

		memo.insert(13, 0, TransFlag::EXACT, 0, None);
		assert_eq!(Some(&Transposition{value: 0, flag: TransFlag::EXACT, depth: 0, best: None, age: 2}), memo.get(&13));
			
		assert_eq!(0, memo.clean());
		assert_eq!(Some(&Transposition{value: 0, flag: TransFlag::EXACT, depth: 0, best: None, age: 2}), memo.get(&13));

		assert_eq!(1, memo.clean());
		assert_eq!(None, memo.get(&13));
	}
}