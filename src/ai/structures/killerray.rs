use crate::backend::movement::*;

//Basert på Killer heurestic: https://en.wikipedia.org/wiki/Killer_heuristic
pub struct Killerray{
	arr: [Option<Move>; SIZE]
}

const SIZE: usize = 20;

impl Killerray{
	pub fn new() -> Self{
		Killerray{arr: [None; SIZE]}
	}

	pub fn get(&self, i: usize) -> &Option<Move>{
		&self.arr[i % SIZE]
	}

	pub fn put(&mut self, m: Move, i: usize){
		self.arr[i % SIZE] = Some(m);
	}
}

#[cfg(test)]
mod killer_testing{
	use super::*;

	#[test]
	fn can_put_and_get(){
		let mut kill = Killerray::new();

		assert_eq!(&None, kill.get(5));

		let m = Move::new(0, 0, 0, 0, None, 0);
		kill.put(m.clone(), 5);

		assert_eq!(&Some(m), kill.get(5));
	}

	#[test]
	fn can_go_out_of_bounds(){
		let mut kill = Killerray::new();

		let m = Move::new(0, 0, 0, 0, None, 0);

		kill.put(m.clone(), 99999999);

		assert_eq!(&Some(m), kill.get(99999999));
	}
}