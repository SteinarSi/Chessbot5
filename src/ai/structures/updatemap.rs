use std::collections::LinkedList;
use crate::backend::board::{Score, Move};

pub struct UpdateMap{
	arr: [LinkedList<Transposition>; SIZE],
	delete_age: u8
}

pub struct Transposition{
	hash: Key,
	value: Score,
	depth: usize,
	flag: TransFlag,
	best: Option<Move>,
	age: u8
}

pub enum TransFlag{
	EXACT,
	UPPER_BOUND,
	LOWER_BOUND
}

const SIZE: usize = 5000000;
const DELETE_LIMIT: u8 = 3;
const NEW_LIST: LinkedList<Transposition> = LinkedList::new();

pub type Key = usize;

impl UpdateMap{
	pub fn new() -> Self{
		UpdateMap{delete_age: 0, arr: [NEW_LIST; SIZE]}
	}

	pub fn update_age(&mut self){
		self.delete_age = (self.delete_age + 1) % 4;
	}

	pub fn insert(&mut self, k: Key, item: Transposition){
		self.arr[k % SIZE].push_front(item);
		/*for i in 0..list.len(){
			if list[i].hash == key{
				list[i] = item;
				return;
			}
		}*/
	}
}