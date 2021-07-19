use std::collections::HashMap;

pub type Key = i64;

pub struct Key_Count{
	map: HashMap<Key, u8>
}

impl Key_Count{
	pub fn new() -> Self{
		Key_Count{map: HashMap::with_capacity(200)}
	}

	pub fn inc(&mut self, k: Key){
		*self.map.entry(k).or_insert(0) += 1;
	}

	pub fn dec(&mut self, k: Key){
		*self.map.get_mut(&k).expect("Expected key entry, but none where found.") -= 1;
	}

	pub fn count(&mut self, k: Key) -> u8{
		*self.map.get(&k).expect("Expected key entry, but none where found.")
	}
}

#[cfg(test)]
mod key_count_tests{
	use super::*;

	#[test]
	fn can_inc(){
		let mut count = Key_Count::new();	
		count.inc(42);
		assert_eq!(1, count.count(42));
	}

	#[test]
	fn can_dec(){
		let mut count = Key_Count::new();
		count.inc(42);
		count.inc(42);
		count.inc(42);

		count.dec(42);
		assert_eq!(2, count.count(42));
	}

	#[should_panic]
	#[test]
	fn cannot_get_unknown_key(){
		Key_Count::new().count(42);
	}
}