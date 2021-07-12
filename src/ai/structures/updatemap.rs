use std::collections::LinkedList;
use crate::backend::board::{Score, Move};

pub struct UpdateMap{
	arr: [Option<Box<Transposition>>; SIZE],
	delete_age: bool
}

#[derive(PartialEq, Debug)]
pub struct Transposition{
	hash: Key,
	value: Score,
	depth: usize,
	flag: TransFlag,
	best: Option<Move>,
	age: bool
}

#[derive(PartialEq, Debug)]
pub enum TransFlag{
	EXACT,
	UPPER_BOUND,
	LOWER_BOUND
}

const SIZE: usize = 2_000_000;
const EMPTY: Option<Box<Transposition>> = None;

pub type Key = usize;

impl UpdateMap{
	pub fn new() -> Self{
		UpdateMap{delete_age: false, arr: [EMPTY; SIZE]}
	}

	pub fn transpose(&self, k: Key, value: Score, depth: usize, flag: TransFlag, best: Option<Move>) -> Transposition{
		Transposition{age: ! self.delete_age, hash: k, value, depth, flag, best}
	}

	pub fn update_age(&mut self){
		self.delete_age = ! self.delete_age;
	}

	pub fn insert(&mut self, item: Transposition){
		let k = item.hash;
		for i in 0..=3{
			if let Some(t) = &self.arr[(k + i) % SIZE]{
				if t.hash == k || t.age == self.delete_age {
					self.arr[(k + i) % SIZE] = Some(Box::new(item));
					return;
				}
			}
		}
		println!("Couldn't find a spot, replacing old entry.");
		self.arr[k % SIZE] = Some(Box::new(item));
	}
	
	pub fn get(&mut self, k: Key) -> Option<&Box<Transposition>>{
		for i in 0..=3{
			if let Some(t) = &self.arr[(k + i) % SIZE]{
				return Some(t);
			}
		}
		None
	}
}


#[cfg(test)]
mod map_tests{
	use super::*;

	#[test]
	fn can_extract(){
		let mut memo = UpdateMap::new();
		let t1 = memo.transpose(13, 50, 3, TransFlag::EXACT, None);
		let t2 = memo.transpose(13, 50, 3, TransFlag::EXACT, None);

		memo.insert(t1);
		assert_eq!(Some(&Box::new(t2)), memo.get(13));
	}
}