use std::collections::HashMap;

pub type Key = i64;

#[derive(Clone)]
pub struct RepetitionCounter{
	map: HashMap<Key, u8>
}

impl RepetitionCounter{
	pub fn new() -> Self{
		RepetitionCounter{map: HashMap::with_capacity(200)}
	}

	pub fn inc(&mut self, k: Key){
		*self.map.entry(k).or_insert(0) += 1;
	}

	pub fn dec(&mut self, k: Key){
		*self.map.get_mut(&k).expect("Expected key entry, but none where found.") -= 1;
	}

	pub fn count(&self, k: Key) -> u8{
		*self.map.get(&k).expect("Expected key entry, but none where found.")
	}
}

//Bryr meg faktisk fint lite om denne er 'korrekt' eller ikke.
//Vi trenger den kun for å vite om Board1==Board2, 
//og da har vi mange andre felt det er mer passende å sjekke for likhet.
impl PartialEq for RepetitionCounter{
	fn eq(&self, _other: &RepetitionCounter) -> bool{
		true
	}
}

#[cfg(test)]
mod key_count_tests{
	use super::*;

	#[test]
	fn can_inc(){
		let mut count = RepetitionCounter::new();	
		count.inc(42);
		assert_eq!(1, count.count(42));
	}

	#[test]
	fn can_dec(){
		let mut count = RepetitionCounter::new();
		count.inc(42);
		count.inc(42);
		count.inc(42);

		count.dec(42);
		assert_eq!(2, count.count(42));
	}

	#[should_panic]
	#[test]
	fn cannot_get_unknown_key(){
		RepetitionCounter::new().count(42);
	}
}