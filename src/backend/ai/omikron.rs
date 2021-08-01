use crate::backend::board_representation::{movement::*, board::*};
use super::structures::{memomap::*, killerray::*, database::*};
use super::interface::AI;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const INITIAL_DEPTH: usize = 99; //Dybden er irrelevant, bortsett fra når vi tester.
const INITIAL_TIME: Duration = Duration::from_secs(10);

pub struct Omikron{
	memo: MemoMap,
	depth: usize,
	killerray: Killerray,
	time: Duration,
	database: Database
}

impl AI for Omikron{
	fn new() -> Self{
		Omikron{memo: MemoMap::new(), depth: INITIAL_DEPTH, killerray: Killerray::new(), time: INITIAL_TIME, database: Database::new()}
	}

	fn set_depth(&mut self, depth: usize){
		self.depth = depth;
	}

	fn search(&mut self, mut b: Board) -> Move{
		match self.database.get(&mut b){
			Some(m) => m,
			None    => {
				let tm = &mut self.memo as *mut MemoMap;
				unsafe {
					multisearch(&mut b, Arc::new(Mutex::new(&mut *tm)), &mut self.killerray, &mut self.time)
				}
				/*
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
		        //let ret =self.memo.get(&b.hash()).unwrap().best.unwrap();
		        //self.memo = MemoMap::new();
		      	//ret
		      	*/
			}
		}
	}
}

fn multisearch(b: &mut Board, mut map: Arc<Mutex<&'static mut MemoMap>>, killerray: &Killerray, stop: &Duration) -> Move{
	let mut d = Arc::new(Mutex::new(2));
	let start = Instant::now();
	let mut handles = Vec::new();
	for i in &[1, 1, 1, 1, 2, 2, 3, 4]{
		let mut tb = b.clone();
		let mut tk = killerray.clone();
		let mut tm = Arc::clone(&map);
		let mut td = Arc::clone(&d);
		let tstop = stop.clone();
		let handle = thread::spawn(move || {
			while start.elapsed() < tstop{
				let depth = *td.lock().unwrap() + i;
				println!("Starter et søk på {}+{}={}", depth-i, i, depth);
				if tb.color_to_move() == White{
					maximize_alpha(&mut tb, - INFINITY, INFINITY, depth, &mut tm, &mut tk, &start, &tstop);
				}else {
					minimize_beta(&mut tb, - INFINITY, INFINITY, depth, &mut tm, &mut tk, &start, &tstop);
				}

				if start.elapsed() < tstop{
					println!("Fullførte et søk på {}+{}={}", depth-i, i, depth);
					let mut od = td.lock().unwrap();
					if depth > *od{
						*od = depth;
					}
				}else{
					println!("Søk avbrutt på {}+{}={}", depth-i, i, depth);
				}
			}
		});
		handles.push(handle);
	}
	
	for handle in handles{
		handle.join().unwrap();
	}
	println!("Høyeste fullførte: {}", d.lock().unwrap());

	map.lock().expect("Couldn't get arc value").get(&b.hash()).expect("This position hasn't been saved").best.expect("This position has no best move saved, despite being a PV node")
}


impl Omikron{
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

	fn quimax(&mut self, b: &mut Board, beta: Score) -> Score{
		//Om den nåværende scoren allerede er uakseptabel for svart kan vi anta at hvits neste trekk gjør den enda mer uakseptabel.
		//Da er det ikke vits i å sjekke engang. Dette antar altså et hvit ikke er i zugswang.
		let stand_pat = b.heuristic_value();
		if stand_pat >= beta && ! b.is_check() { return stand_pat; } 

		let ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

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


	fn quimin(&mut self, b: &mut Board, alpha: Score) -> Score{
		let stand_pat = b.heuristic_value();
		if stand_pat <= alpha && ! b.is_check() { return stand_pat; }

		let ms = b.moves();
		if ms.len() == 0 { return b.end_score(); }

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

	fn maximize_alpha(&mut self, b: &mut Board, mut alpha: Score, mut beta: Score, depth: usize, time: &Instant) -> Score{
		if b.is_draw_by_repetition() { 
			self.memo.insert(b.hash(), 0, TransFlag::EXACT, 999, None);
			return 0; 
		}
		if depth <= 1 { return self.quimax(b, beta); }

		let mut exact = false;
		let mut best = None;
		let mut prev = None;
		let mut kill = None;
		let mut bestscore = - INFINITY;

		//Sjekker om vi allerede har et relevant oppslag i hashmappet vårt.
		if let Some(t) = self.memo.get(&b.hash()){
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
			else if let Some(m) = t.best{
				if b.is_legal(&m) { 
					prev = t.best;
					b.move_piece(&m);
					bestscore = self.minimize_beta(b, alpha, beta, depth-1, time);
					b.go_back();

					//Beta-cutoff, stillingen er uakseptabel for svart.
					if bestscore >= beta{
						self.killerray.put(m.clone(), b.counter());
						self.memo.insert(b.hash(), bestscore, TransFlag::LOWER_BOUND, depth, Some(m));
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
		}

		//Sjekker om vi har et 'Killer Move' for denne dybden.
		//Da evaluerer vi det trekket først.
		if let Some(mut m) = self.killerray.get(b.counter()){
			if b.is_legal(&m) && Some(m) != prev{
				m.set_heuristic_value(b.value_of(&m));
				kill = Some(m);
				b.move_piece(&m);
				let value = self.minimize_beta(b, alpha, beta, depth-1, time);
				b.go_back();

				if value > bestscore{
					//Beta-cutoff, stillingen er uakseptabel for svart.
					if value >= beta{
						self.memo.insert(b.hash(), bestscore, TransFlag::LOWER_BOUND, depth, Some(m));
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
		if ms.len() == 0 { return b.end_score(); }

		ms.sort_by_heuristic(White);
		let mut iter = ms.into_iter();

		if prev == None && kill == None{
			let m = iter.next().unwrap();
			b.move_piece(&m);
			let value = self.minimize_beta(b, alpha, beta, depth-1, time);
			b.go_back();
			if value > bestscore{
				if value > alpha{
					if value >= beta{
						self.killerray.put(m.clone(), b.counter());
						self.memo.insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
						return value;
					}
					alpha = value;
					best = Some(m);
					exact = true;
				}
				bestscore = value;
			}
		}

		for m in iter{
			if depth >= 6 && time.elapsed() >= self.time { return bestscore; } 
			if Some(m) == prev || Some(m) == kill { continue; }
			b.move_piece(&m);
			let mut value = self.minimize_beta(b, alpha, alpha+1, depth-1, time); //Null window
			if value > alpha && value < beta {
				value = self.minimize_beta(b, alpha, beta, depth-1, time); //Re-search
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
					self.killerray.put(m.clone(), b.counter());
					self.memo.insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
					return value;
				}
			}
		}

		self.memo.insert(b.hash(), alpha, if exact {TransFlag::EXACT} else { TransFlag::UPPER_BOUND }, depth, best);
		bestscore
	}

	fn minimize_beta(&mut self, b: &mut Board, mut alpha: Score, mut beta: Score, depth: usize, time: &Instant) -> Score{
		if b.is_draw_by_repetition() { 
			self.memo.insert(b.hash(), 0, TransFlag::EXACT, 999, None);
			return 0; 
		}
		if depth <= 1 { return self.quimin(b, alpha); }

		let mut exact = false;
		let mut best = None;
		let mut prev = None;
		let mut kill = None;
		let mut bestscore = INFINITY;

		if let Some(t) = self.memo.get(&b.hash()){
			if t.depth >= depth{
				match &t.flag{
					TransFlag::EXACT       => { return t.value; }
					TransFlag::LOWER_BOUND => { if t.value >= beta { return t.value; } }
					TransFlag::UPPER_BOUND => { if t.value <= alpha { return t.value; }}
				}
			}
			else if let Some(m) = t.best{
				if b.is_legal(&m){
					prev = t.best;
					b.move_piece(&m);
					bestscore = self.maximize_alpha(b, alpha, beta, depth-1, time);
					b.go_back();

					if bestscore <= alpha{
						self.killerray.put(m, b.counter());
						self.memo.insert(b.hash(), bestscore, TransFlag::UPPER_BOUND, depth, Some(m));
						return bestscore;
					}

					if bestscore < beta{
						exact = true;
						beta = bestscore;
						best = Some(m);
					}
				}
			}
		}

		if let Some(mut m) = self.killerray.get(b.counter()){
			if b.is_legal(&m) && Some(m) != prev{
				m.set_heuristic_value(b.value_of(&m));
				kill = Some(m);
				b.move_piece(&m);
				let value = self.maximize_alpha(b, alpha, beta, depth-1, time);
				b.go_back();

				if value < bestscore{
					if value <= alpha{
						self.memo.insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
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
		if ms.len() == 0 { return b.end_score(); }
		ms.sort_by_heuristic(Black);
		let mut iter = ms.into_iter();

		if prev.is_none() && kill.is_none(){
			let m = iter.next().unwrap();
			b.move_piece(&m);
			let value = self.maximize_alpha(b, alpha, beta, depth-1, time);
			b.go_back();

			if value < bestscore{
				if value < beta{
					if value <= alpha{
						self.killerray.put(m.clone(), b.counter());
						self.memo.insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
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
			if depth >= 6 && time.elapsed() >= self.time { return bestscore; }
			if Some(m) == prev || Some(m) == kill { continue; }
			b.move_piece(&m);
			let mut value = self.maximize_alpha(b, beta-1, beta, depth-1, time);
			if value > alpha && value < beta{
				value = self.maximize_alpha(b, alpha, beta, depth-1, time);
				if value < beta{
					beta = value;
					exact = true;
					best = Some(m);
				}
			}
			b.go_back();
			if value < bestscore{
				if value <= alpha{
					self.killerray.put(m, depth);
					self.memo.insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
					return value;
				}
				bestscore = value;
			}
		}
		self.memo.insert(b.hash(), beta, if exact { TransFlag::EXACT } else { TransFlag::LOWER_BOUND }, depth, best);
		bestscore
	}
}
fn maximize_alpha(b: &mut Board, mut alpha: Score, beta: Score, depth: usize, mut map: &mut Arc<Mutex<&mut MemoMap>>, killerray: &mut Killerray, start: &Instant, stop: &Duration) -> Score{
	if b.is_draw_by_repetition() { 
		map.lock().expect("Couldn't get arc value").insert(b.hash(), 0, TransFlag::EXACT, 999, None);
		return 0; 
	}
	if depth <= 1 { return quimax(b, beta); }

	let mut exact = false;
	let mut best = None;
	let mut prev = None;
	let mut kill = None;
	let mut bestscore = - INFINITY;

	//Sjekker om vi allerede har et relevant oppslag i hashmappet vårt.
	if let Some(t) = map.lock().expect("Couldn't get arc value").get(&b.hash()){
		if t.depth >= depth{
			//Hvis dybden på oppslaget er nok, kan vi bruke verdiene umiddelbart.
			match &t.flag{
				TransFlag::EXACT       => { return t.value; }
				TransFlag::LOWER_BOUND => { if t.value >= beta { return t.value; } }
				TransFlag::UPPER_BOUND => { if t.value <= alpha { return t.value; }}
			}
		}
		prev = t.best;
		//Hvis ikke kan vi ikke bruke verdiene.
		//Da sjekker vi istedet om oppslaget har lagret et bra trekk, og evaluerer i så fall det trekket først.
	}
	if let Some(m) = prev{
		if b.is_legal(&m) {
			b.move_piece(&m);
			bestscore = minimize_beta(b, alpha, beta, depth-1, map, killerray, start, stop);
			b.go_back();

			//Beta-cutoff, stillingen er uakseptabel for svart.
			if bestscore >= beta{
				killerray.put(m.clone(), b.counter());
				map.lock().expect("Couldn't get arc value").insert(b.hash(), bestscore, TransFlag::LOWER_BOUND, depth, Some(m));
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
			let value = minimize_beta(b, alpha, beta, depth-1, map, killerray, start, stop);
			b.go_back();

			if value > bestscore{
				//Beta-cutoff, stillingen er uakseptabel for svart.
				if value >= beta{
					map.lock().expect("Couldn't get arc value").insert(b.hash(), bestscore, TransFlag::LOWER_BOUND, depth, Some(m));
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
	if ms.len() == 0 { return b.end_score(); }

	ms.sort_by_heuristic(White);
	let mut iter = ms.into_iter();

	if prev == None && kill == None{
		let m = iter.next().unwrap();
		b.move_piece(&m);
		let value = minimize_beta(b, alpha, beta, depth-1, map, killerray, start, stop);
		b.go_back();
		if value > bestscore{
			if value > alpha{
				if value >= beta{
					killerray.put(m.clone(), b.counter());
					map.lock().expect("Couldn't get arc value").insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
					return value;
				}
				alpha = value;
				best = Some(m);
				exact = true;
			}
			bestscore = value;
		}
	}

	for m in iter{
		if depth >= 6 && &start.elapsed() >= stop { return bestscore; } 
		if Some(m) == prev || Some(m) == kill { continue; }
		b.move_piece(&m);
		let mut value = minimize_beta(b, alpha, alpha+1, depth-1, map, killerray, start, stop); //Null window
		if value > alpha && value < beta {
			value = minimize_beta(b, alpha, beta, depth-1, map, killerray, start, stop); //Re-search
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
				map.lock().expect("Couldn't get arc value").insert(b.hash(), value, TransFlag::LOWER_BOUND, depth, Some(m));
				return value;
			}
		}
	}

	map.lock().expect("Couldn't get arc value").insert(b.hash(), alpha, if exact {TransFlag::EXACT} else { TransFlag::UPPER_BOUND }, depth, best);
	bestscore
}

fn minimize_beta(b: &mut Board, mut alpha: Score, mut beta: Score, depth: usize, mut map: &mut Arc<Mutex<&mut MemoMap>>, killerray: &mut Killerray, start: &Instant, stop: &Duration) -> Score{
	if b.is_draw_by_repetition() { 
		map.lock().expect("Couldn't get arc value").insert(b.hash(), 0, TransFlag::EXACT, 999, None);
		return 0; 
	}
	if depth <= 1 { return quimin(b, alpha); }

	let mut exact = false;
	let mut best = None;
	let mut prev = None;
	let mut kill = None;
	let mut bestscore = INFINITY;

	if let Some(t) = map.lock().expect("Couldn't get arc value").get(&b.hash()){
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
			bestscore = maximize_alpha(b, alpha, beta, depth-1, map, killerray, start, stop);
			b.go_back();

			if bestscore <= alpha{
				killerray.put(m, b.counter());
				map.lock().expect("Couldn't get arc value").insert(b.hash(), bestscore, TransFlag::UPPER_BOUND, depth, Some(m));
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
			let value = maximize_alpha(b, alpha, beta, depth-1, map, killerray, start, stop);
			b.go_back();

			if value < bestscore{
				if value <= alpha{
					map.lock().expect("Couldn't get arc value").insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
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
	if ms.len() == 0 { return b.end_score(); }
	ms.sort_by_heuristic(Black);
	let mut iter = ms.into_iter();

	if prev.is_none() && kill.is_none(){
		let m = iter.next().unwrap();
		b.move_piece(&m);
		let value = maximize_alpha(b, alpha, beta, depth-1, map, killerray, start, stop);
		b.go_back();

		if value < bestscore{
			if value < beta{
				if value <= alpha{
					killerray.put(m.clone(), b.counter());
					map.lock().expect("Couldn't get arc value").insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
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
		if depth >= 6 && &start.elapsed() >= stop { return bestscore; }
		if Some(m) == prev || Some(m) == kill { continue; }
		b.move_piece(&m);
		let mut value = maximize_alpha(b, beta-1, beta, depth-1, map, killerray, start, stop);
		if value > alpha && value < beta{
			value = maximize_alpha(b, alpha, beta, depth-1, map, killerray, start, stop);
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
				map.lock().expect("Couldn't get arc value").insert(b.hash(), value, TransFlag::UPPER_BOUND, depth, Some(m));
				return value;
			}
			bestscore = value;
		}
	}
	map.lock().expect("Couldn't get arc value").insert(b.hash(), beta, if exact { TransFlag::EXACT } else { TransFlag::LOWER_BOUND }, depth, best);
	bestscore
}

fn quimax(b: &mut Board, beta: Score) -> Score{
	//Om den nåværende scoren allerede er uakseptabel for svart kan vi anta at hvits neste trekk gjør den enda mer uakseptabel.
	//Da er det ikke vits i å sjekke engang. Dette antar altså et hvit ikke er i zugswang.
	let stand_pat = b.heuristic_value();
	if stand_pat >= beta && ! b.is_check() { return stand_pat; } 

	let ms = b.moves();
	if ms.len() == 0 { return b.end_score(); }

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

fn quimin(b: &mut Board, alpha: Score) -> Score{
	let stand_pat = b.heuristic_value();
	if stand_pat <= alpha && ! b.is_check() { return stand_pat; }

	let ms = b.moves();
	if ms.len() == 0 { return b.end_score(); }

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

#[cfg(test)]
mod omikron_tests{
	use super::*;

	#[ignore]
	#[test]
	fn bot_takes_win_if_possible(){
		let s = "\
k-------
------R-
-------R
--------
--------
--------
--------
-------K";
		let mut b = Board::custom(s, White);
		let mut bot = Omikron::new();

		let expected = Move::from_str("h6h8").unwrap();
		let actual = bot.search(b.clone());
		assert!(b.is_legal(&expected));

		assert!(expected.from == actual.from);
		assert!(expected.to == actual.to);
	}

	#[ignore]
	#[test]
	//This should be a mate in a few moves, let's see if the bot actually gets it.
	fn bot_should_win_this(){
		let s = "\
--------
--------
--k-----
------R-
-------R
--------
--------
-------K";
		let b = Board::custom(s, White);
		let mut bot = Omikron::new();
		bot.search(b.clone());
		panic!("{:?}", bot.principal_variation(b));
	}

	#[ignore]
	#[test]
	fn promotion_mate(){
		let s = "\
-k------
--R----p
----N---
--KP----
--------
--------
--------
--------";	
		let mut bot = Omikron::new();
		let b = Board::custom(s, White);
		bot.search(b.clone());
		panic!("{:?}", bot.principal_variation(b));	
	}

	#[ignore]
	#[test]
	fn pls(){
	let s = "\
--------
--------
--------
--------
--------
-----k--
p--r----
-----K--";
		let mut bot = Omikron::new();
		let m = bot.search(Board::custom(s, Black));

		assert!(&[String::from("a2a1"), String::from("d2d1")].contains(&m.to_string_short()));
	}
}