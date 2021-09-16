use crate::backend::board_representation::{movement::*, board::*};
use super::structures::{memomap::*, killerray::*, database::*};
use super::interface::AI;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const INITIAL_DEPTH: usize = 99; //Dybden er irrelevant, bortsett fra når vi tester.
const INITIAL_TIME: Duration = Duration::from_secs(10);
const MEMO_LIMIT: usize = 2;

pub struct Splitter{
	memo: MemoMap,
	depth: usize,
	killerray: Killerray,
	time: Duration,
	database: Database
}

impl AI for Splitter{
	fn new() -> Self{
		Splitter{memo: MemoMap::new(), depth: INITIAL_DEPTH, killerray: Killerray::new(), time: INITIAL_TIME, database: Database::new()}
	}

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}
	fn search(&mut self, mut b: Board) -> Move{
		match self.database.get(&mut b){
			Some(m) => m,
			None    => {
				panic!();
				/*
				let time = Instant::now();
				let mut d = 2;
				let tm = &mut self.memo as *mut MemoMap;
				unsafe{
					while time.elapsed() < self.time && d <= self.depth{
						if b.color_to_move() == White{
							maximize_alpha(&mut b, &mut Arc::new(Mutex::new(&mut *tm)), &mut self.killerray, - INFINITY, INFINITY, d, &time, &self.time, true);
						}
						else{
							minimize_beta(&mut b, &mut Arc::new(Mutex::new(&mut *tm)), &mut self.killerray, - INFINITY, INFINITY, d, &time, &self.time, true);
						}
						d += 1;
					}
				}
				println!("Depth reached: {}", d-1);
				println!("{:?}", self.principal_variation(b.clone()));
				let mem = self.memo.get(&b.hash()).unwrap();
				println!("Expected value: {}, {:?}", mem.value, mem.flag);
				self.memo.clean();
		        self.memo.get(&b.hash()).unwrap().best.unwrap()*/
			}
		}
	}
/*
	fn search(&mut self, mut b: Board) -> Move{
		match self.database.get(&mut b){
			Some(m) => m,
			None    => {
				let time = Instant::now();
				let mut d = 2;
				while time.elapsed() < self.time && d <= self.depth {
					if b.color_to_move() == White{
						self.maximize_alpha(&mut b, - INFINITY, INFINITY, d, &time);
					}
					else{
						self.minimize_beta(&mut b, - INFINITY, INFINITY, d, &time);
					}
					//if let Some(t) = self.memo.get(&b.hash()).unwrap().best{
					//	println!("Best so far, at depth {}: {}", d, t.to_string_short());
					//}
					d += 1;
				}
				println!("Depth reached: {}", d-1);
				println!("{:?}", self.principal_variation(b.clone()));
				let mem = self.memo.get(&b.hash()).unwrap();
				println!("Expected value: {}, {:?}", mem.value, mem.flag);
				self.memo.clean();
		        self.memo.get(&b.hash()).unwrap().best.unwrap()
			}
		}
	}
	*/
}

impl Splitter{
	pub fn set_time(&mut self, seconds: u64){
		self.time = Duration::from_secs(seconds);
	}

	pub fn principal_variation(&mut self, mut b: Board) -> Moves{
		let mut ret = Moves::new();
		loop{
			match self.memo.get(&b.hash()){
				None => { break; }
				Some(m) => {
					match m.best{
						None => { break; }
						Some(m) => {
							b.move_piece(&m);
							ret.push(m);
							if b.is_draw_by_repetition() { break; }
						}
					}
				}
			}
		}
		if b.is_checkmate() { print!("Checkmate is imminent: "); }
		ret

	}
}

fn quimax(b: &mut Board, map: &mut Arc<Mutex<&mut MemoMap>>, beta: Score) -> Score{
	//Om den nåværende scoren allerede er uakseptabel for svart kan vi anta at hvits neste trekk gjør den enda mer uakseptabel.
	//Da er det ikke vits i å sjekke engang. Dette antar altså et hvit ikke er i zugswang.
	let stand_pat = b.heuristic_value();
	if stand_pat >= beta && ! b.is_check() { return stand_pat; } 

	let ms = b.moves();
	if ms.len() == 0 { 
		let s = b.end_score();
		map.lock().expect("Couldn't get arc value").insert(b.hash(), s, TransFlag::EXACT, 999, None);
		return s;
	}

	//Filtrerer trekk, vi vil kun ha trekk som flytter til en rute som ikke er truet.
	let mut arr = [[None; 8]; 8];
	let filt = |m: &Move| {
		if let Some(bo) = arr[m.to.y][m.to.x]{
			bo
		}else{
			let bo = ! b.is_threatened(&m.to);
			arr[m.to.y][m.to.x] = Some(bo);
			bo
		}
	};

	let mut ms: Moves = ms.into_iter().filter(filt).collect();
	if ms.len() == 0 { stand_pat }
	else {
		ms.sort_by_heuristic(White);
		b.heuristic_value() + ms[0].heuristic_value()
	}
}


fn quimin(b: &mut Board, map: &mut Arc<Mutex<&mut MemoMap>>, alpha: Score) -> Score{
	let stand_pat = b.heuristic_value();
	if stand_pat <= alpha && ! b.is_check() { return stand_pat; }

	let ms = b.moves();
	if ms.len() == 0 {
		let s = b.end_score();
		map.lock().expect("Couldn't get arc value").insert(b.hash(), s, TransFlag::EXACT, 999, None);
		return s; 
	}

	let mut arr = [[None; 8]; 8];
	let filt = |m: &Move| {
		if let Some(bo) = arr[m.to.y][m.to.x]{
			bo
		}else{
			let bo = ! b.is_threatened(&m.to);
			arr[m.to.y][m.to.x] = Some(bo);
			bo
		}
	};

	let mut ms: Moves = ms.into_iter().filter(filt).collect();
	if ms.len() == 0 { stand_pat }
	else {
		ms.sort_by_heuristic(Black);
		b.heuristic_value() + ms[0].heuristic_value()
	}

}

fn maximize_alpha(b: &'static mut Board, map: &'static mut Arc<Mutex<&mut MemoMap>>, killerray: &mut Killerray, mut alpha: Score, beta: Score, depth: usize, time: &Instant, stop: &Duration, is_pv: bool) -> Score{
	if b.is_draw_by_repetition() { 
		map.lock().expect("Couldn't get arc value.").insert(b.hash(), 0, TransFlag::EXACT, 999, None);
		return 0; 
	}
	if depth <= 1 { return quimax(b, map, beta); }

	let mut exact = false;
	let mut best = None;
	let mut prev = None;
	let mut kill = None;
	let mut bestscore = - INFINITY;

	//Sjekker om vi allerede har et relevant oppslag i hashmappet vårt.
	if let Some(t) = map.lock().expect("Couldn't get arc value.").get(&b.hash()){
		if t.depth >= depth{
			//Hvis dybden på oppslaget er nok, kan vi bruke verdiene umiddelbart.
			match &t.flag{
				TransFlag::EXACT       => { return t.value; }
				TransFlag::LOWER_BOUND => { if t.value >= beta { return t.value; } }
				TransFlag::UPPER_BOUND => { if t.value <= alpha { return t.value; }}
			}
		}
		//Hvis ikke kan vi ikke bruke verdiene.
		//Da sjekker vi istedet om oppslaget har lagret et bra trekk, og evaluerer i så fall det trekket først.
		prev = t.best;
	}
	if let Some(m) = prev {
		if b.is_legal(&m) {
			b.move_piece(&m);
			bestscore = minimize_beta(b, map, killerray, alpha, beta, depth-1, time, stop, is_pv);
			b.go_back();

			//Beta-cutoff, stillingen er uakseptabel for svart.
			if bestscore >= beta{
				killerray.put(m.clone(), b.counter());
				map.lock().expect("Couldn't get arc value.").insert(b.hash(), bestscore, TransFlag::LOWER_BOUND, depth, Some(m));
				return bestscore;
			}

			//Trekket forbedret alfa, dette er dermed en PV-node.
			if bestscore > alpha{
				best = Some(m);
				exact = true;
				alpha = bestscore;
			}
		}
	}

	//Sjekker om vi har et 'Killer Move' for denne dybden.
	//Da evaluerer vi det trekket først.
	if let Some(mut m) = killerray.get(b.counter()){
		if b.is_legal(&m) && Some(m) != prev{
			m.set_heuristic_value(b.value_of(&m));
			kill = Some(m);
			b.move_piece(&m);
			let value = minimize_beta(b, map, killerray, alpha, beta, depth-1, time, stop, is_pv);
			b.go_back();

			if value > bestscore{
				//Beta-cutoff, stillingen er uakseptabel for svart.
				if value >= beta{
					map.lock().expect("Couldn't get arc value.").insert(b.hash(), bestscore, TransFlag::LOWER_BOUND, depth, Some(m));
					return value;
				}

				//Trekket forbedret alfa.
				if value > alpha{
					best = Some(m);
					exact = true;
					alpha = value;
				}
				bestscore = value;
			}
		}
	}

	// Først her, når vi allerede har vurdert eventuelle 'Killer'-trekk og memoiserte trekk,
	// begynner vi å generere listen av alle trekk og evaluerer dem.
	let mut ms = b.moves();
	if ms.len() == 0 { 
		let s = b.end_score();
		map.lock().expect("Couldn't get arc value").insert(b.hash(), s, TransFlag::EXACT, 999, None);
		return s;
	}

	ms.sort_by_heuristic(White);
	let mut iter = ms.into_iter();

	//Valigivs søker vi et memoisert trekk, eller et killer-trekk først. Men om vi ikke har noen av dem,
	//får et helt vanlig trekk prioritet istedet. Det betyr at det søkes på <alpha, beta>, istedet for <alpha, alpha+1> som resten.
	if prev.is_none() && kill.is_none(){
		let m = iter.next().unwrap();
		b.move_piece(&m);
		let value = minimize_beta(b, map, killerray, alpha, beta, depth-1, time, stop, is_pv);
		b.go_back();
		if value > bestscore{
			if value > alpha{
				if value >= beta{
					killerray.put(m.clone(), b.counter());
					map.lock().expect("Couldn't get arc value.").insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
					return value;
				}
				alpha = value;
				best = Some(m);
				exact = true;
			}
			bestscore = value;
		}
	}

	if is_pv{
		/*let mut child_count: u8 = 0;
		let (mut tx, mut rx) = mpsc::channel();
		for child in iter{
			if depth >= 6 && &time.elapsed() >= stop { return bestscore; } 
			if Some(child) == prev || Some(child) == kill { continue; }
			child_count += 1;
			let mut cmap = &mut Arc::clone(&map);
			let mut ckiller = &mut killerray.clone();
			let mut cb = &mut b.clone();
			let mut ctx = tx.clone();
			let ctime = time.clone();
			let cstop = stop.clone();
			let calpha = alpha;
			let cbeta = beta;
			let m = child.clone();
			let cdepth = depth;
			thread::spawn(move || {
				b.move_piece(&m);
				let mut value = minimize_beta(cb, cmap, ckiller, calpha, calpha+1, cdepth-1, &ctime, &cstop, false);
				if value > calpha && value < cbeta{
					value = minimize_beta(cb, cmap, ckiller, calpha, cbeta, cdepth-1, &ctime, &cstop, false);
				}
				ctx.send((value, m)).unwrap();
			});
		}
		if child_count > 0{
			let (bestvalue, bestmove) = (0..child_count).map(|_| rx.recv().unwrap()).max_by(|(v1, _), (v2, _)| v1.cmp(v2)).unwrap();
			if bestvalue > bestscore{
				map.lock().expect("Couldn't get arc value.").insert(b.hash(), bestvalue, TransFlag::EXACT, depth, Some(bestmove));
				return bestvalue;
			}
		}
		map.lock().expect("Couldn't get arc value.").insert(b.hash(), bestscore, TransFlag::EXACT, depth, best);*/
		return bestscore;
	}else{
		for m in iter{
			if depth >= 6 && &time.elapsed() >= stop { return bestscore; } 
			if Some(m) == prev || Some(m) == kill { continue; }
			b.move_piece(&m);
			let mut value = minimize_beta(b, map, killerray, alpha, alpha+1, depth-1, time, stop, false); //Null window
			if value > alpha && value < beta {
				value = minimize_beta(b, map, killerray, alpha, beta, depth-1, time, stop, false); //Re-search
				if value > alpha{
					alpha = value;
					exact = true;
					best = Some(m);
				}
			}
			b.go_back();

			if value > bestscore{
				bestscore = value;
				if value >= beta{
					killerray.put(m.clone(), b.counter());
					map.lock().expect("Couldn't get arc value.").insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
					return value;
				}
			}
		}
		map.lock().expect("Couldn't get arc value.").insert(b.hash(), alpha, if exact {TransFlag::EXACT} else { TransFlag::UPPER_BOUND }, depth, best);
		bestscore
	}
}

fn minimize_beta(b: &'static mut Board, map: &'static mut Arc<Mutex<&mut MemoMap>>, killerray: &mut Killerray, alpha: Score, mut beta: Score, depth: usize, time: &Instant, stop: &Duration, is_pv: bool) -> Score{
	if b.is_draw_by_repetition() { 
		map.lock().expect("Couldn't get arc value.").insert(b.hash(), 0, TransFlag::EXACT, 999, None);
		return 0; 
	}
	if depth <= 1 { return quimin(b, map, alpha); }

	let mut exact = false;
	let mut best = None;
	let mut prev = None;
	let mut kill = None;
	let mut bestscore = INFINITY;

	if let Some(t) = map.lock().expect("Couldn't get arc value.").get(&b.hash()){
		if t.depth >= depth{
			match &t.flag{
				TransFlag::EXACT       => { return t.value; }
				TransFlag::LOWER_BOUND => { if t.value >= beta { return t.value; } }
				TransFlag::UPPER_BOUND => { if t.value <= alpha { return t.value; }}
			}
		}
		prev = t.best;
	}
	if let Some(m) = prev{
		if b.is_legal(&m){
			b.move_piece(&m);
			bestscore = maximize_alpha(b, map, killerray, alpha, beta, depth-1, time, stop, is_pv);
			b.go_back();

			if bestscore <= alpha{
				killerray.put(m, b.counter());
				map.lock().expect("Couldn't get arc value.").insert(b.hash(), bestscore, TransFlag::UPPER_BOUND, depth, Some(m));
				return bestscore;
			}

			if bestscore < beta{
				exact = true;
				beta = bestscore;
				best = Some(m);
			}
		}
	}
	
	if let Some(mut m) = killerray.get(b.counter()){
		if b.is_legal(&m) && Some(m) != prev{
			m.set_heuristic_value(b.value_of(&m));
			kill = Some(m);
			b.move_piece(&m);
			let value = maximize_alpha(b, map, killerray, alpha, beta, depth-1, time, stop, is_pv);
			b.go_back();

			if value < bestscore{
				if value <= alpha{
					map.lock().expect("Couldn't get arc value.").insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
					return value;
				}

				bestscore = value;
				if value < beta{
					exact = true;
					beta = value;
					best = Some(m);
				}
			}
		}
	}

	let mut ms = b.moves();
	if ms.len() == 0 { 
		let s = b.end_score();
		map.lock().expect("Couldn't get arc value").insert(b.hash(), s, TransFlag::EXACT, 999, None);
		return s; 
	}
	ms.sort_by_heuristic(Black);
	let mut iter = ms.into_iter();

	if prev.is_none() && kill.is_none(){
		let m = iter.next().unwrap();
		b.move_piece(&m);
		let value = maximize_alpha(b, map, killerray, alpha, beta, depth-1, time, stop, is_pv);
		b.go_back();

		if value < bestscore{
			if value < beta{
				if value <= alpha{
					killerray.put(m.clone(), b.counter());
					map.lock().expect("Couldn't get arc value.").insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
					return value;
				}
				beta = value;
				exact = true;
				best = Some(m);
			}
			bestscore = value;
		}
	}

	for m in iter{
		if depth >= 6 && &time.elapsed() >= stop { return bestscore; }
		if Some(m) == prev || Some(m) == kill { continue; }
		b.move_piece(&m);
		let mut value = maximize_alpha(b, map, killerray, beta-1, beta, depth-1, time, stop, false);
		if value > alpha && value < beta{
			value = maximize_alpha(b, map, killerray, alpha, beta, depth-1, time, stop, false);
			if value < beta{
				beta = value;
				exact = true;
				best = Some(m);
			}
		}
		b.go_back();
		if value < bestscore{
			if value <= alpha{
				killerray.put(m, depth);
				map.lock().expect("Couldn't get arc value.").insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
				return value;
			}
			bestscore = value;
		}
	}
	map.lock().expect("Couldn't get arc value.").insert(b.hash(), beta, if exact { TransFlag::EXACT } else { TransFlag::LOWER_BOUND }, depth, best);
	bestscore
}