use std::collections::HashMap;
use crate::backend::board_representation::board::{Move, Score};

pub type Key = i64;
const LIFETIME: u8 = 3;
const ADD: u8 = LIFETIME - 1;


//En wrapper rundt HashMap, for å memoisere tidligere evaluerte posisjoner.
//Mellom hvert botsøk må vi kalle .clean() for å fjerne utdaterte oppslag.
//'Utdatert' betyr oppslag som ikke har blitt lest siden forrige gang .clean() ble kallt.
pub struct MemoMap{
	map: HashMap<Key, Transposition>,
	//delete: u8

	delete: bool
}

//Holder styr på informasjon vi har lyst til å lagre for hver tidligere evaluerte posisjon.
#[derive(PartialEq, Debug)]
pub struct Transposition{
	pub value: Score,
	pub flag: TransFlag,
	pub depth: usize,
	pub best: Option<Move>,	
	age: bool
}

// 🏳️‍⚧️
#[derive(PartialEq, Debug)]
pub enum TransFlag{ 
	EXACT,
	UPPER_BOUND,
	LOWER_BOUND
}

impl MemoMap{
	pub fn new() -> Self{
		MemoMap{map: HashMap::with_capacity(2_000_000), delete: true}
	}

	pub fn get(&mut self, k: &Key) -> Option<&Transposition>{
		match self.map.get_mut(k){
			None => None,
			Some(t) => { t.age = ! self.delete; Some(t) }
		}
	}

	pub fn insert(&mut self, k: Key, value: Score, flag: TransFlag, depth: usize, best: Option<Move>){
		if let Some(t) = self.map.get(&k){
			if t.depth >= depth { return; }
		}
		self.map.insert(k, Transposition{value, flag, depth, best, age: ! self.delete});
	}

	pub fn clean(&mut self) -> usize{
		let before = self.map.len();
		let delete = self.delete;
		self.map.retain(|_, v| (*v).age != delete);
		self.delete = ! self.delete;

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

		assert_eq!(Some(&Transposition{value: 0, flag: TransFlag::EXACT, depth: 0, best: None, age: false}), memo.get(&13));
	}

	#[test]
	fn cleaning_twice_removes_entry(){
		let mut memo = MemoMap::new();

		memo.insert(13, 0, TransFlag::EXACT, 0, None);
		assert_eq!(Some(&Transposition{value: 0, flag: TransFlag::EXACT, depth: 0, best: None, age: false}), memo.get(&13));

		memo.clean();
		memo.clean();
		assert_eq!(None, memo.get(&13));
	}

	#[test]
	fn getting_resets_timer(){
		let mut memo = MemoMap::new();

		memo.insert(13, 0, TransFlag::EXACT, 0, None);

		for _ in 0..10{
			memo.clean();
			assert!(memo.get(&13).is_some());
		}
	}
}
