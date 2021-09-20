use crate::backend::board_representation::board::*;
use std::{env, fs, io::Write, collections::{HashMap, HashSet}};
use std::io::LineWriter;

pub struct Database{
	map: HashMap<i64, Moves>
}

impl Database{
	pub fn new() -> Self{
		let mut map = HashMap::new();
		let file = match fs::read_to_string("openings.zobrist"){
			Ok(f) => f,
			Err(_)    => {
				create_database();
				fs::read_to_string("openings.zobrist").expect("I created the file, but still can't find it")
			}
		};
		for mut line in file.split("\n").map(|l| l.trim_end().split(" ")){
			let key = match line.next(){
				None    => break,
				Some(k) => match k.parse(){
					Err(_) => break,
					Ok(k)  => k
				}
			};

			//let key = line.next().unwrap().parse().unwrap_or_else(|_| break);//(&format!("Could not parse key"));
			map.insert(key, Moves::new());
			for m in line{
				map.get_mut(&key).unwrap().push(Move::from_str(m).expect(&format!("Could not parse: {}", m)));
			}
		}
		Database{map}
	}

	pub fn get(&self, b: &mut Board) -> Option<Move>{
		let mut m = self.map.get(&b.hash())?.choice();
		if ! b.is_legal(&m) { return None; }
		m.set_heuristic_value(b.value_of(&m));
		Some(m)
	}
}



//Første gangen dette programmet kjøres på en masking blir denne kalt opp.
//Den parser openings.txt til et map på i64 -> Moves, og skriver det til en ny fil.
fn create_database(){
	let mut db: HashMap<i64, HashSet<Move>> = HashMap::new();
	let fil = fs::read_to_string("openings.txt").expect("Could not find the file.");
	for line in fil.split("\n"){
		let moves = line.trim_end().split(" ").map(|m| Move::from_str(m).expect(&format!("Could not parse {}", m)));
		let mut b = Board::new();
		for m in moves.into_iter(){
			let hash = b.hash();
			b.move_piece(&m);
			db.entry(hash).or_insert(HashSet::new()).insert(m);
		}
	}

	let mut file = fs::File::create("openings.zobrist").unwrap();
	for key in db.keys(){
		file.write_all(key.to_string().as_bytes()).expect("Failed when writing opening book.");
		for m in db.get(&key).unwrap(){
			file.write_all(format!(" {}", m.to_string_short()).as_bytes()).expect("Failed when writing opening book.");
		}
		file.write_all("\n".as_bytes()).expect("Failed when writing opening book.");
	}
}

#[cfg(test)]
mod database_tests{
	use super::*;

	#[test]
	fn can_open_database(){
		let d = Database::new();
		assert!(d.map.len() > 0);
	}

	#[test]
	fn can_get_move(){
		let d = Database::new();
		assert!(d.get(&mut Board::new()).is_some());
	}

	#[test]
	fn database_contains_good_moves_only(){
		let mut b = Board::new();
		let d = Database::new();
		b.move_str("c2c4").unwrap();
		b.move_str("e7e5").unwrap();
		b.move_str("b1c3").unwrap();

		let expected: Moves = ["g8f6", "f8b4", "d7d6", "b8c6", "f7f5"].iter().map(|m| Move::from_str(m).unwrap()).collect();

		//Den velger et tilfeldig trekk, så la oss kalle denne mange ganger for sikkerhets skyld
		assert!(expected.contains(&d.get(&mut b).unwrap()));
		assert!(expected.contains(&d.get(&mut b).unwrap()));
		assert!(expected.contains(&d.get(&mut b).unwrap()));
		assert!(expected.contains(&d.get(&mut b).unwrap()));
		assert!(expected.contains(&d.get(&mut b).unwrap()));
		assert!(expected.contains(&d.get(&mut b).unwrap()));
	}
}


