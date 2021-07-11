use std::collections::LinkedList;
use crate::backend::board::{Score, Move};

pub struct UpdateMap{
	arr: [Vec<Transposition>; SIZE],
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

const SIZE: usize = 10000;
const NEW_LIST: Vec<Transposition> = Vec::new();

pub type Key = usize;

impl UpdateMap{
	pub fn new() -> Self{
		UpdateMap{delete_age: false, arr: [NEW_LIST; SIZE]}
	}

	pub fn transpose(&self, k: Key, value: Score, depth: usize, flag: TransFlag, best: Option<Move>) -> Transposition{
		Transposition{age: ! self.delete_age, hash: k, value, depth, flag, best}
	}

	pub fn update_age(&mut self){
		self.delete_age = ! self.delete_age;
	}

	pub fn insert(&mut self, item: Transposition){
		let vec = &mut self.arr[item.hash % SIZE];
		let mut i = 0;
		while i < vec.len(){
			let t = &vec[i];
			if t.hash == item.hash{
				vec[i] = item;
				return;
			}
			if t.age == self.delete_age{
				vec.remove(i);
			}else{
				i += 1;
			}
		}
		vec.push(item);
	}
	
	pub fn get(&mut self, k: Key) -> Option<&Transposition>{
		let mut i = 0;
		let vec = &mut self.arr[k % SIZE];
		while i < vec.len(){
			let t = &vec[i];
			if t.hash == k{
				return Some(&self.arr[k % SIZE][0]);
			}
			if t.age == self.delete_age{
				vec.remove(i);
			}else{
				i += 1;
			}
		}
		None
	}

	fn size_at(&self, k: Key) -> usize{
		self.arr[k % SIZE].len()
	}
}


#[cfg(test)]
mod map_tests{
	use super::*;

	#[test]
	fn can_insert(){
		let mut memo = UpdateMap::new();
		let t = memo.transpose(13, 50, 3, TransFlag::EXACT, None);

		assert_eq!(0, memo.size_at(13));
		memo.insert(t);
		assert_eq!(1, memo.size_at(13));
	}

	#[test]
	fn can_extract(){
		let mut memo = UpdateMap::new();
		let t1 = memo.transpose(13, 50, 3, TransFlag::EXACT, None);
		let t2 = memo.transpose(13, 50, 3, TransFlag::EXACT, None);

		memo.insert(t1);
		assert_eq!(Some(&t2), memo.get(13));
	}

}