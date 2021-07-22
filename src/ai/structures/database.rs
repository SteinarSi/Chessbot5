use crate::backend::board::*;
use std::{env, fs, io::Write, collections::{HashMap, HashSet}};
use std::io::LineWriter;

struct CreateDataBase{
	map: HashMap<i64, HashSet<Move>>
}

pub fn create_database(){
	let mut db: HashMap<i64, HashSet<Move>> = HashMap::new();
	let fil = fs::read_to_string("openings.txt").expect("Could not find the file.");
	for line in fil.split("\n"){
		let moves = line.trim_right().split(" ").map(|m| Move::from_str(m).expect(&format!("Could not parse {}", m)));
		let mut b = Board::new();
		for m in moves.into_iter(){
			let hash = b.hash();
			b.move_piece(&m);
			db.entry(hash).or_insert(HashSet::new()).insert(m);
		}
	}

	let mut file = fs::File::create("openings.zobrist").unwrap();
	for key in db.keys(){
		file.write_all(key.to_string().as_bytes());
		for m in db.get(&key).unwrap(){
			file.write_all(format!(" {}", m.to_string()).as_bytes());
		}
		file.write_all("\n".as_bytes());
	}
}