pub use super::piece::{Piece, PieceType, Color, Color::White, Color::Black};
pub use super::movement::{Move, Score};
use super::zobrist::Zobrist;
use super::movement::{Position, INFINITY};
use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

const BOARD_SIZE: usize = 8;
const DEFAULT_BOARD: &str = "\
rnbqkbnr
pppppppp
--------
--------
--------
--------
PPPPPPPP
RNBQKBNR";

#[derive(PartialEq, Clone)]
pub struct Board{
	grid: [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE],
	color_to_move: Color,
	scores: Vec<Score>,
	counter: usize,
	graveyard: Vec<TombStone>,
	moves: Vec<Move>,
	passants: Vec<Option<i8>>,
	castles: Vec<Castle>,
	wkingpos: Vec<Position>,
	bkingpos: Vec<Position>,
	hashes: Vec<i64>,
	zobrist: Zobrist

	//TODO blir mange flere felt her etter hvert.
}

//Holder styr på hvor og når brikker blir tatt, så de kan respawnes etterpå.
#[derive(PartialEq, Clone)]
struct TombStone{
	piece: Piece,
	date: usize,
	position: Position
}
//             w short|w long|b short|b long
type Castle = (bool,   bool,  bool,   bool);

impl Board{
	pub fn new() -> Self{
		Board::custom(DEFAULT_BOARD, White)
	}

	pub fn move_piece(&mut self, m: &Move){
		let pie = self.get_clone_at(&m.from).unwrap();
		if let Some(target) = self.get_clone_at(&m.to){
			self.zobrist.update_pos(&m.to, &target);
			self.graveyard.push(TombStone{piece: target, position: m.to, date: self.counter})
		}
		self.grid[m.from.y][m.from.x] = None;
		self.grid[m.to.y][m.to.x] = Some(pie);
		self.zobrist.update_pos(&m.from, &pie);
		self.zobrist.update_pos(&m.to, &pie);

		self.zobrist.remove_en_passant();
		let passant;
		if pie.piecetype == PieceType::Pawn{
			passant = self.handle_pawn_moves(m);
		}else{
			passant = None;
		}
		if pie.piecetype == PieceType::King{
			self.move_rook_if_castling(m, false);
		}
		if pie.piecetype == PieceType::King{
			if self.color_to_move == White { self.wkingpos.push(m.to); self.bkingpos.push(self.bkingpos[self.counter]);}
			else { self.bkingpos.push(m.to); self.wkingpos.push(self.wkingpos[self.counter]); }
		}else { 
			self.wkingpos.push(self.wkingpos[self.counter]); 
			self.bkingpos.push(self.bkingpos[self.counter]); 
		}
		self.zobrist.swap_sides();
		self.hashes.push(self.zobrist.hash());
		self.scores.push(self.scores[self.counter] + m.heurestic_value());
		self.update_castle(m);
		self.passants.push(passant);
		self.moves.push(*m);
		self.counter += 1;
		self.color_to_move = self.color_to_move.opposite();
	}

	pub fn go_back(&mut self){
		let m = self.moves.pop().expect("Cannot go further back!");
		self.hashes.pop();
		self.scores.pop();
		self.castles.pop();
		self.wkingpos.pop();
		self.bkingpos.pop();
		self.counter -= 1;
		self.zobrist.set_hash(self.hashes[self.counter]);
		self.color_to_move = self.color_to_move.opposite();
		self.passants.pop();

		let pie = self.get_clone_at(&m.to);
		if pie.unwrap().piecetype == PieceType::King{
			self.move_rook_if_castling(&m, true);
		}
		self.grid[m.from.y][m.from.x] = pie;
		self.grid[m.to.y][m.to.x] = None;

		let d = self.graveyard.len();
		if d > 0{
			let ts = &self.graveyard[d-1];
			if ts.date == self.counter{
				self.grid[ts.position.y][ts.position.x] = Some(ts.piece);
				self.graveyard.pop();
			}
		}
	}

	//Genererer en liste av lovlige trekk.
	pub fn moves(&mut self) -> Vec<Move>{
		let mut ret = Vec::new();
		let color = self.color_to_move;

		for y in 0..BOARD_SIZE{
			for x in 0..BOARD_SIZE{
				if let Some(p) = self.get_clone_at(&Position{x, y}){
					if p.color == color{
						if p.piecetype == PieceType::Pawn{
							ret.append(&mut self.pawn_moves(x, y, &p));
						}
						else if p.piecetype == PieceType::King{
							ret.append(&mut self.king_moves(x, y, &p));
						}
						else if p.can_run(){
							ret.append(&mut self.running_moves(x, y, &p));
						}
						else{
							ret.append(&mut self.knight_moves(x, y, &p));
						}
					}
				}
			}
		}
		ret = ret.into_iter().filter(|m| self.is_not_check_if(m)).collect();
		ret.append(&mut self.castle_moves());
		ret
	}

	pub fn move_str(&mut self, s: &str) -> Option<()>{
		let m1 = Move::from_str(s)?;
		let m2 = self.moves().into_iter().find(|m2| *m2 == m1)?;
		self.move_piece(&m2);
		Some(())
	}

	pub fn is_legal(&mut self, m: Move) -> bool{
		self.moves().contains(&m)
	}

	pub fn heurestic_value(&self) -> Score{
		self.scores[self.counter]
	}

	pub fn color_to_move(&self) -> Color{
		self.color_to_move
	}

	pub fn counter(&self) -> usize{
		self.counter
	}

	pub fn hash(&self) -> i64{
		self.zobrist.hash()
	}

	pub fn is_checkmate(&mut self) -> bool{
		self.moves().len() == 0
	}

	pub fn winner(&self) -> Option<Color>{
		match self.end_score().cmp(&0){
			Ordering::Less => Some(Black),
            Ordering::Greater => Some(White),
            Ordering::Equal => None
		}
	}

	//NB!!!! Denne må kun kalles i botsøket, etter at moves.len() == 0.
	pub fn end_score(&self) -> Score{
		if self.color_to_move == White{
			if self.is_threatened_by(&self.wkingpos[self.counter], Black){
				- INFINITY + self.counter as Score //Sjakk matt, svart vant
			}
			else { 0 } //Patt
		}
		else{
			if self.is_threatened_by(&self.bkingpos[self.counter], White){
				INFINITY - self.counter as Score //Sjakk matt, hvit vant
			}
			else { 0 } //Patt
		}
	}

	//Sjekker om et trekk leder til at kongen står i sjakk eller ikke.
	fn is_not_check_if(&mut self, m: &Move) -> bool{
		let pie = self.get_clone_at(&Position{x: m.from.x, y: m.from.y});
		let target = self.get_clone_at(&Position{x: m.to.x, y: m.to.y});
		self.grid[m.from.y][m.from.x] = None;
		self.grid[m.to.y][m.to.x] = pie;
		let ret;
		let kpos = if pie.unwrap().piecetype == PieceType::King { m.to } 
				   else { 
					   if self.color_to_move == White { self.wkingpos[self.counter] }
					   else { self.bkingpos[self.counter] } };
		if self.color_to_move == White{
			ret = ! self.is_threatened_by(&kpos, Black);
		}else{
			ret = ! self.is_threatened_by(&kpos, White);
		}
		self.grid[m.from.y][m.from.x] = pie;
		self.grid[m.to.y][m.to.x] = target;

		ret
	}

	pub fn is_threatened(&self, pos: &Position) -> bool{
		self.is_threatened_by(pos, self.color_to_move.opposite())
	}

	fn is_threatened_by(&self, pos: &Position, color: Color) -> bool{
		for dir in &[(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)]{
			let to_x = pos.x as i8 + dir.0;
			let to_y = pos.y as i8 + dir.1;
			if to_x < 0 || to_x >= 8 || to_y < 0 || to_y >= 8 { continue; }
			if let Some(p) = self.get_reference_at(to_x as usize, to_y as usize){
				if p.piecetype == PieceType::Knight{
					if p.color == color{
						return true;
					}
				}
			}
		}
		'outer: for dir in &[(0, -1), (1, 0), (0, 1), (-1, 0)]{
			let mut to_x = pos.x as i8;
			let mut to_y = pos.y as i8;
			let mut first_step = true;
			loop{
				to_x += dir.0;
				to_y += dir.1;
				if to_x < 0 || to_x >= 8 || to_y < 0 || to_y >= 8 { continue 'outer; }
				if let Some(p) = self.get_reference_at(to_x as usize, to_y as usize){
					if p.color == color && (p.piecetype == PieceType::Rook || p.piecetype == PieceType::Queen || (p.piecetype == PieceType::King && first_step)){
						return true;
					}
					break;
				}

				first_step = false;
			}
		}
		'outer: for dir in &[(1, 1), (-1, 1)]{
			let mut to_x = pos.x as i8;
			let mut to_y = pos.y as i8;
			let mut first_step = true;
			loop{
				to_x += dir.0;
				to_y += dir.1;
				if to_x < 0 || to_x >= 8 || to_y < 0 || to_y >= 8 { continue 'outer; }
				if let Some(p) = self.get_reference_at(to_x as usize, to_y as usize){
					if p.color == color && (p.piecetype == PieceType::Bishop || p.piecetype == PieceType::Queen || p.piecetype == PieceType::King || (p.piecetype == PieceType::Pawn && p.color == White && first_step)){
						return true;
					}
					break;
				}
				first_step = false;
			}
		}
		'outer: for dir in &[(1, -1), (-1, -1)]{
			let mut to_x = pos.x as i8;
			let mut to_y = pos.y as i8;
			let mut first_step = true;
			loop{
				to_x += dir.0;
				to_y += dir.1;
				if to_x < 0 || to_x >= 8 || to_y < 0 || to_y >= 8 { continue 'outer; }
				if let Some(p) = self.get_reference_at(to_x as usize, to_y as usize){
					if p.color == color && (p.piecetype == PieceType::Bishop || p.piecetype == PieceType::Queen || p.piecetype == PieceType::King || (p.piecetype == PieceType::Pawn && p.color == Black && first_step)){
						return true;
					}
					break;
				}
				first_step = false;
			}
		}
		false
	}

	//Takler alle spesialtilfeller når en bonde skal flyttes.
	//Det inkluderer en passant, og promotering.
	fn handle_pawn_moves(&mut self, m: &Move) -> Option<i8>{
		let passant;
		if (m.from.y as i8 - m.to.y as i8).abs() == 2{
			passant = Some(m.from.x as i8);
			self.zobrist.update_en_passant(m.from.x);
		}else { 
			passant = None; 
		}

		if let Some(ps) = self.passants[self.counter]{
			if m.to.x == ps as usize {
				if m.to.y == 2 && self.color_to_move == White {
					let pos = Position{x: m.to.x, y: 3};
					let target = self.get_clone_at(&pos).unwrap();
					self.graveyard.push(TombStone{piece: target, position: pos, date: self.counter});
					self.grid[3][m.to.x] = None;
					self.zobrist.update_pos(&pos, &target);
				}
				else if m.to.y == 5 && self.color_to_move == Black {
					let pos = Position{x: m.to.x, y: 4};
					let target = self.get_clone_at(&pos).unwrap();
					self.graveyard.push(TombStone{piece: target, position: pos, date: self.counter});
					self.grid[4][m.to.x] = None;
					self.zobrist.update_pos(&pos, &target);
				}
			}
		}

		if m.promote.is_some(){
			self.zobrist.update_pos(&m.to, &self.get_reference_at(m.to.x, m.to.y).unwrap());
			self.grid[m.to.y][m.to.x] = m.promote;
			self.zobrist.update_pos(&m.to, &m.promote.unwrap());
		}

		passant
	}

	//Kongetrekk og hestetrekk, dvs brikker som ikke kan flytte mer enn et skritt om gangen.
	fn knight_moves(&self, x: usize, y: usize, p: &Piece) -> Vec<Move>{
		let mut ret = Vec::new();
		let color = self.color_to_move;
		for dir in p.directions(){
			let to_x = x as i8 + dir.0;
			let to_y = y as i8 + dir.1;
			let from_pos = Position{x, y};
			let to_pos = Position{x: to_x as usize, y: to_y as usize};
			let from_value = p.value_at(&from_pos);
			if to_x < 0 || to_x > 7 || to_y < 0 || to_y > 7 { continue; }
			if let &Some(t) = self.get_reference_at(to_x as usize, to_y as usize) {
				if t.color != color { 
					ret.push(Move::new(x, y, to_x as usize, to_y as usize, None, - t.combined_value_at(&to_pos) + p.value_at(&to_pos) - from_value));
				} else { continue; }
			}
			ret.push(Move::new(x, y, to_x as usize, to_y as usize, None, p.value_at(&to_pos) - from_value));
		}
		ret
	}

	fn king_moves(&mut self, x: usize, y: usize, p: &Piece) -> Vec<Move>{
		let mut ret = Vec::new();
		let color = self.color_to_move;
		self.grid[y][x] = None;
		for dir in p.directions(){
			let to_x = x as i8 + dir.0;
			let to_y = y as i8 + dir.1;
			let from_pos = Position{x, y};
			let to_pos = Position{x: to_x as usize, y: to_y as usize};
			let from_value = p.value_at(&from_pos);
			if to_x < 0 || to_x > 7 || to_y < 0 || to_y > 7 { continue; }
			if let &Some(t) = self.get_reference_at(to_x as usize, to_y as usize) {
				if t.color != color { 
					ret.push(Move::new(x, y, to_x as usize, to_y as usize, None, - t.combined_value_at(&to_pos) + p.value_at(&to_pos) - from_value));
				} else { continue; }
			}
			ret.push(Move::new(x, y, to_x as usize, to_y as usize, None, p.value_at(&to_pos) - from_value));
		}
		self.grid[y][x] = Some(*p);
		ret
	}

	//Trekkene til alle brikker som kan gå flere skritt om gangen.
	fn running_moves(&self, x: usize, y: usize, p: &Piece) -> Vec<Move>{
		let mut ret = Vec::new();
		let color = self.color_to_move;
		let from_pos = Position{x, y};
		let from_value = p.value_at(&from_pos);
		for dir in p.directions(){
			let mut to_x = x as i8 + dir.0;
			let mut to_y = y as i8 + dir.1;
			let mut to_pos;
			loop{
				to_pos = Position{x: to_x as usize, y: to_y as usize};
				if to_x < 0 || to_x > 7 || to_y < 0 || to_y > 7 { break; }
				if let &Some(t) = self.get_reference_at(to_x as usize, to_y as usize) {
					if t.color != color { 
						ret.push(Move::new(x, y, to_x as usize, to_y as usize, None, - t.combined_value_at(&to_pos) + p.value_at(&to_pos) - from_value)); 
					}
					break;
				}
				ret.push(Move::new(x, y, to_x as usize, to_y as usize, None, p.value_at(&to_pos) - from_value));
				to_x += dir.0;
				to_y += dir.1;
			}
		}
		ret
	}

	//Alle bondetrekk
	fn pawn_moves(&self, x: usize, y: usize, p: &Piece) -> Vec<Move>{
		let mut ret = Vec::new();
		let from_pos = Position{x, y};
		let from_value = p.value_at(&from_pos);
		for dir in p.directions(){
			let to_y = y as i8 + dir.1;
			if dir.0 == 0 {
				let to_pos = Position{x, y: to_y as usize};
				if to_y < 8 && to_y >= 0 && self.get_reference_at(x, to_y as usize) == &None{
					if to_y == 0 {
						let Q = Piece::new('Q');
						let N = Piece::new('N');
						ret.push(Move::new(x, y, x, to_y as usize, Q, Q.unwrap().combined_value_at(&to_pos) - p.combined_value_at(&from_pos)));
						ret.push(Move::new(x, y, x, to_y as usize, N, N.unwrap().combined_value_at(&to_pos) - p.combined_value_at(&from_pos)));
					} else if to_y == 7 {
						let q = Piece::new('q');
						let n = Piece::new('n');
						ret.push(Move::new(x, y, x, to_y as usize, q, q.unwrap().combined_value_at(&to_pos) - p.combined_value_at(&from_pos)));
						ret.push(Move::new(x, y, x, to_y as usize, n, n.unwrap().combined_value_at(&to_pos) - p.combined_value_at(&from_pos)));
					}
					else { ret.push(Move::new(x, y, x, to_y as usize, None, p.value_at(&to_pos) - from_value)); }

					if y == 6 && self.color_to_move == White && self.get_reference_at(x, (to_y-1) as usize) == &None{
						ret.push(Move::new(x, y, x, (to_y - 1) as usize, None, p.value_at(&Position{x, y: (to_y - 1) as usize}) - from_value));
					} else if y == 1 && self.color_to_move == Black && self.get_reference_at(x, (to_y+1) as usize) == &None{
						ret.push(Move::new(x, y, x, (to_y + 1) as usize, None, p.value_at(&Position{x, y: (to_y + 1) as usize}) - from_value));
					}
				}
			}
			else{
				let to_x = x as i8 + dir.0;
				let to_pos = Position{x: to_x as usize, y: to_y as usize};
				if to_x >= 0 && to_x < 8{
					if let &Some(t) = self.get_reference_at(to_x as usize, to_y as usize) {
						if t.color != self.color_to_move { 
							if to_y == 0 {
								let Q = Piece::new('Q');
								let N = Piece::new('N');
								ret.push(Move::new(x, y, to_x as usize, to_y as usize, Q, Q.unwrap().combined_value_at(&to_pos) - p.combined_value_at(&from_pos) - t.combined_value_at(&to_pos)));
								ret.push(Move::new(x, y, to_x as usize, to_y as usize, N, N.unwrap().combined_value_at(&to_pos) - p.combined_value_at(&from_pos) - t.combined_value_at(&to_pos)));
							} else if to_y == 7{
								let q = Piece::new('q');
								let n = Piece::new('n');
								ret.push(Move::new(x, y, to_x as usize, to_y as usize, q, q.unwrap().combined_value_at(&to_pos) - p.combined_value_at(&from_pos) - t.combined_value_at(&to_pos)));
								ret.push(Move::new(x, y, to_x as usize, to_y as usize, n, n.unwrap().combined_value_at(&to_pos) - p.combined_value_at(&from_pos) - t.combined_value_at(&to_pos)));
							}
							else{
								ret.push(Move::new(x, y, to_x as usize, to_y as usize, None, p.value_at(&to_pos) - from_value - t.combined_value_at(&to_pos)));
							}
						}
					} else if let Some(c) = self.passants[self.counter]{
						if c == to_x && (self.color_to_move == White && y == 3 || self.color_to_move == Black && y == 4){
							let passant_pos = Position{x: c as usize, y};
							ret.push(Move::new(x, y, to_x as usize, to_y as usize, None, p.value_at(&to_pos) - from_value - self.get_reference_at(passant_pos.x, passant_pos.y).unwrap().combined_value_at(&passant_pos))); //TODO
						}
					}
				}
			}
		}
		ret
	}

	fn move_rook_if_castling(&mut self, m: &Move, back: bool){
		if back{
			match (m.from.x, m.from.y, m.to.x, m.to.y) {
				(4, 7, 6, 7) => { self.grid[7].swap(7, 5); },
				(4, 7, 2, 7) => { self.grid[7].swap(0, 3); },
				(4, 0, 6, 0) => { self.grid[0].swap(7, 5); },
				(4, 0, 2, 0) => { self.grid[0].swap(0, 3); }
				_ => {}
			}
		}	
		else{
			match (m.from.x, m.from.y, m.to.x, m.to.y) {
				(4, 7, 6, 7) => { 
				let rw = self.get_reference_at(7, 7).unwrap();
				self.zobrist.update_xy(7, 7, &rw);
				self.zobrist.update_xy(5, 7, &rw);
				self.grid[7].swap(7, 5); },
			(4, 7, 2, 7) => { 
				let rw = self.get_reference_at(0, 7).unwrap();
				self.zobrist.update_xy(0, 7, &rw);
				self.zobrist.update_xy(3, 7, &rw);
				self.grid[7].swap(0, 3); },
			(4, 0, 6, 0) => { 
				let rb = self.get_reference_at(7, 0).unwrap();
				self.zobrist.update_xy(7, 0, &rb);
				self.zobrist.update_xy(5, 0, &rb);
				self.grid[0].swap(7, 5); },
			(4, 0, 2, 0) => { 
				let rb = self.get_reference_at(0, 0).unwrap();
				self.zobrist.update_xy(0, 0, &rb);
				self.zobrist.update_xy(3, 0, &rb);
				self.grid[0].swap(0, 3); },
			_ => {}
			}
		}
	}

	fn get_clone_at(&self, p: &Position) -> Option<Piece>{
		self.grid[p.y][p.x].clone()
	}

	fn get_reference_at(&self, x: usize, y: usize) -> &Option<Piece>{
		&self.grid[y][x]
	}

	fn update_castle(&mut self, m: &Move){
		let c = self.castles[self.counter as usize];
		let next = (c.0 && m.from != E1 && m.from != H1 && m.to != H1,
					c.1 && m.from != E1 && m.from != A1 && m.to != A1,
					c.2 && m.from != E8 && m.from != H8 && m.to != H8,
					c.3 && m.from != E8 && m.from != A8 && m.to != A8);
		self.zobrist.update_castle((c.0 != next.0, c.1 != next.1, c.2 != next.2, c.3 != next.3));
		self.castles.push(next);
	}


	fn castle_moves(&self) -> Vec<Move>{
		let mut ret = Vec::new();
		let castle = self.castles[self.counter as usize];
		if self.color_to_move == White{
			if castle.0 && self.get_reference_at(5, 7).is_none() && self.get_reference_at(6, 7).is_none() &&
				[Position{x: 4, y: 7}, Position{x: 5, y: 7}, Position{x: 6, y: 7}].into_iter().all(|p| ! self.is_threatened_by(&p, Black)){
				ret.push(Move::new(4, 7, 6, 7, None, 49));
			}
			if castle.1 && self.get_reference_at(1, 7).is_none() && self.get_reference_at(2, 7).is_none() && self.get_reference_at(3, 7).is_none() &&
			[Position{x: 4, y: 7}, Position{x: 3, y: 7}, Position{x: 2, y: 7}].into_iter().all(|p| ! self.is_threatened_by(&p, Black)) {
				ret.push(Move::new(4, 7, 2, 7, None, 40));
			}
		}
		else{
			if castle.2 && self.get_reference_at(5, 0).is_none() && self.get_reference_at(6, 0).is_none() &&
			[Position{x: 4, y: 0}, Position{x: 5, y: 0}, Position{x: 6, y: 0}].into_iter().all(|p| ! self.is_threatened_by(&p, White)) {
				ret.push(Move::new(4, 0, 6, 0, None, -49));
			}
			if castle.3 && self.get_reference_at(1, 0).is_none() && self.get_reference_at(2, 0).is_none() && self.get_reference_at(3, 0).is_none() &&
			[Position{x: 4, y: 0}, Position{x: 3, y: 0}, Position{x: 2, y: 0}].into_iter().all(|p| ! self.is_threatened_by(&p, White)){
				ret.push(Move::new(4, 0, 2, 0, None, -40));
			}
		}
		ret
	}

	//Dette kan forkortes mye i et hvilket som helst annet språk en Rust :(
	fn build_castle(grid: &[[Option<Piece>; 8]; 8]) -> Castle{
		let mut ret = (false, false, false, false);
		if let Some(p) = grid[7][4]{
			if p.piecetype == PieceType::King && p.color == White{
				if let Some(p) = grid[7][7]{
					if p.piecetype == PieceType::Rook && p.color == White { ret.0 = true; }
				}
				if let Some(p) = grid[7][0]{
					if p.piecetype == PieceType::Rook && p.color == White { ret.1 = true; }
				}
			}
		}
		if let Some(p) = grid[0][4]{
			if p.piecetype == PieceType::King && p.color == Black{
				if let Some(p) = grid[0][7]{
					if p.piecetype == PieceType::Rook && p.color == Black { ret.2 = true; }
				}
				if let Some(p) = grid[0][0]{
					if p.piecetype == PieceType::Rook && p.color == Black { ret.3 = true; }
				}
			}
		} 
		ret
	}

	pub fn custom(s: &str, c: Color) -> Self{
		let s = s.replace(&['\n'][..], "");
		let mut grid = [[None; BOARD_SIZE]; BOARD_SIZE];
		let mut y = 0;
		let mut x = 0;
		let mut wk = (100, 100);
		let mut bk = (100, 100);
		for c in s.chars(){
			grid[y][x] = Piece::new(c);
			if let Some(p) = grid[y][x]{
				if p.piecetype == PieceType::King {
					if p.color == White { wk = (x, y); }
					else { bk = (x, y); }
				}
	  		}
			x += 1;
			if x == 8{
				y += 1;
				x = 0;
			}
		}
		let zobrist = Zobrist::new(&grid, c);
		Board{grid, color_to_move: c, counter: 0, graveyard: Vec::new(), 
			moves: Vec::new(), passants: vec![None], castles: vec![Board::build_castle(&grid)],
			scores: vec![0], wkingpos: vec![Position{x: wk.0, y: wk.1}], bkingpos: vec![Position{x: bk.0, y: bk.1}],
		 	hashes: vec![zobrist.hash()], zobrist,}
	}
}

const E1: Position = Position{x: 4, y: 7};
const E8: Position = Position{x: 4, y: 0};
const A1: Position = Position{x: 0, y: 7};
const A8: Position = Position{x: 0, y: 0};
const H1: Position = Position{x: 7, y: 7};
const H8: Position = Position{x: 7, y: 0};

impl ToString for Board{
	fn to_string(&self) -> String{
		let mut ret = String::new();
		for y in 0..BOARD_SIZE{
			for x in 0..BOARD_SIZE{
				if let Some(p) = self.grid[y][x] { 
					ret.push(p.char()); 
				}
				else { ret.push('-'); }
				
			}
			ret.push('\n');
		}
		ret
	}	
}

impl Hash for Board{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.zobrist.hash().hash(state);
	}
}

impl fmt::Debug for Board{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "grid:\n{}counter: {}\ngraveyard: {:?}\n, color: {}\n passants: {:?}\nhashcode: {}", self.to_string(), self.counter, self.graveyard, self.color_to_move, self.passants, self.zobrist.hash())
	}
}

impl fmt::Debug for TombStone{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "turn: {}, piece: {}", self.date, self.piece)
	}
}



///////////////////////////////
//           TESTS           //
///////////////////////////////

const EMPTY: &str = "\
--------
--------
--------
--------
--------
--------
--------
--------";

#[cfg(test)]
mod test_move_generation{
	use super::*;

	#[test]
	fn just_a_king(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[7][4] = Piece::new('K');
		let moves = board.moves();
		let expected: Vec<Move> = ["e1d2", "e1e2", "e1f2", "e1f1", "e1d1"].iter().map(|s| Move::from_str(s).unwrap()).collect();

		assert_eq!(moves, expected);
	}

	#[test]
	fn just_a_knight(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[0][1] = Piece::new('N');
		let actual = board.moves();
		let expected: Vec<Move> = ["b8c6", "b8d7", "b8a6"].iter().map(|s| Move::from_str(s).unwrap()).collect();

		assert_eq!(expected, actual);
	}

	#[test]
	fn just_a_bishop(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[1][1] = Piece::new('B');
		let actual = board.moves();
		let expected: Vec<Move> = ["b7a8", "b7c8", "b7c6", "b7d5", "b7e4", "b7f3", "b7g2", "b7h1", "b7a6"].iter().map(|s| Move::from_str(s).unwrap()).collect();

		assert_eq!(expected, actual);
	}

	#[test]
	fn can_capture_enemy(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[6][0] = Piece::new('p');
		board.grid[6][1] = Piece::new('p');
		board.grid[7][0] = Piece::new('Q');
		let moves = board.moves();

		assert!(moves.contains(&Move::from_str("a1a2").unwrap()));
		assert!(moves.contains(&Move::from_str("a1b2").unwrap()));

		assert!( ! moves.contains(&Move::from_str("a1a3").unwrap())); //Kan ikke hoppe over folk
	}

	#[test]
	fn can_not_capture_own_piece(){
		let mut board = Board::custom(EMPTY, Black);
		board.grid[0][4] = Piece::new('n');
		board.grid[0][3] = Piece::new('q');
		let moves = board.moves();

		assert!( ! moves.contains(&Move::from_str("d8e8").unwrap())); //Kan ikke ta egen brikke
		assert!( ! moves.contains(&Move::from_str("d8f8").unwrap())); //Kan ikke hoppe over egen brikke
		assert!(moves.contains(&Move::from_str("d8c8").unwrap()));   //Kan derimot gjøre vanlige trekk.
	}

	#[test]
	fn just_a_pawn(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[4][3] = Piece::new('P');
		let actual = board.moves();
		let expected = vec![Move::from_str("d4d5").unwrap()];
		assert_eq!(expected, actual);
	}
}

#[cfg(test)]
mod score_test{
	use super::*;

	#[test]
	fn same_moves_should_yield_same_score(){
		let mut board = Board::new();
		assert_eq!(0, board.heurestic_value());

		board.move_str("e2e4");
		board.move_str("e7e5");

		assert_eq!(0, board.heurestic_value());

		board.move_str("b1c3");
		board.move_str("b8c6");

		assert_eq!(0, board.heurestic_value());

		board.move_str("f1c5");
		board.move_str("f8c4");

		assert_eq!(0, board.heurestic_value());
	}
}

#[cfg(test)]
mod threat_test{
	use super::*;

	#[test]
	fn initial_board_threats(){
		let mut board = Board::new();

		//Tilfeldige ruter som svart truer eller passer på
		for (x, y) in &[(1, 2), (2, 2), (4, 2), (6, 2), (7, 2), (0, 1), (1, 1), (2, 1), (2, 0), (5, 1), (7, 1)]{
			assert!(board.is_threatened_by(&Position{x: *x, y: *y}, Black));
		}
		//Noen ruter som svart ikke truer eller passer på
		for (x, y) in &[(0, 0), (7, 0), (3, 3), (1, 6), (4, 6), (4, 5), (7, 6)]{
			assert!( ! board.is_threatened_by(&Position{x: *x, y: *y}, Black));
		}

		for (x, y) in &[(0, 6), (1, 6), (5, 6), (7, 6), (1, 5), (0, 5), (4, 5), (6, 5)]{
			assert!(board.is_threatened_by(&Position{x: *x, y: *y}, White));
		}

		for (x, y) in &[(0, 7), (7, 7), (4, 4), (0, 1), (5, 1), (7, 1)]{
			assert!( ! board.is_threatened_by(&Position{x: *x, y: *y}, White));
		}
	}

	#[test]
	fn cannot_go_into_check(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[0][1] = Piece::new('K');
		board.grid[1][7] = Piece::new('r');

		assert!( ! board.is_legal(Move::from_str("b8b7").unwrap()));
		assert!( ! board.is_legal(Move::from_str("b8a7").unwrap()));
		assert!( ! board.is_legal(Move::from_str("b8c7").unwrap()));

		assert!(board.is_legal(Move::from_str("b8a8").unwrap()));
	}

	#[test]
	fn must_escape_check(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[0][1] = Piece::new('K');
		board.grid[0][7] = Piece::new('r');

		assert!(board.is_legal(Move::from_str("b8b7").unwrap()));
		assert!(board.is_legal(Move::from_str("b8a7").unwrap()));
		assert!(board.is_legal(Move::from_str("b8c7").unwrap()));

		assert!( ! board.is_legal(Move::from_str("b8a8").unwrap()));
		assert!( ! board.is_legal(Move::from_str("b8c8").unwrap()));
	}

	#[test]
	fn can_block_check(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[0][1] = Piece::new('K');
		board.grid[0][7] = Piece::new('r');
		board.grid[4][4] = Piece::new('R');

		assert!(board.is_legal(Move::from_str("e4e8").unwrap()));
	}

	#[test]
	fn fools_mate(){
		let mut board = Board::new();
		board.move_str("f2f3");
		board.move_str("e7e5");
		board.move_str("g2g4");
		board.move_str("d8h4");

		assert!(board.is_checkmate());
		assert_eq!(board.winner(), Some(Black));
	}

	#[test]
	fn scholars_mate(){
		let mut board = Board::new();
		board.move_str("e2e4");
		board.move_str("e7e5");
		board.move_str("f1c4");
		board.move_str("b8c6");
		board.move_str("d1h5");
		board.move_str("g8f6");
		board.move_str("h5f7");

		println!("{}", board.to_string());

		assert!(board.is_checkmate());
		assert_eq!(board.winner(), Some(White));
	}
}

#[cfg(test)]
mod pawn_tests{
	use super::*;

	#[test]
	fn can_go_two_squares(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[6][0] = Piece::new('P');
		let actual = board.moves();
		let expected = vec![Move::from_str("a2a3").unwrap(), Move::from_str("a2a4").unwrap()];

		assert_eq!(expected, actual);
	}

	#[test]
	fn black_pawns_go_backwards(){
		let mut board = Board::custom(EMPTY, Black);
		board.grid[1][4] = Piece::new('p');
		let actual = board.moves();
		let expected = vec![Move::from_str("e7e6").unwrap(), Move::from_str("e7e5").unwrap()];

		assert_eq!(expected, actual);
	}

	#[test]
	fn pawns_capture_diagonally(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[6][1] = Piece::new('P');
		board.grid[5][0] = Piece::new('p'); //Fiendtlig bonde i rekkevidde
		board.grid[5][2] = Piece::new('P'); //Vennlig bonde i rekkevidde, kan ikke ta denne

		let actual = board.moves();
		let expected: Vec<Move> = vec!["c3c4", "b2a3", "b2b3", "b2b4"].iter().map(|s| Move::from_str(s).unwrap()).collect();

		assert_eq!(expected, actual);
	}

	#[test]
	fn en_passant(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[6][7] = Piece::new('P');
		board.grid[4][6] = Piece::new('p');

		board.move_piece(&Move::from_str("h2h4").unwrap());

		let actual = board.moves();
		let expected = vec![Move::from_str("g4g3").unwrap(), Move::from_str("g4h3").unwrap()];

		assert_eq!(expected, actual);
	}

	#[test]
	fn en_croissant(){
		let mut board = Board::custom(EMPTY, Black);
		board.grid[3][4] = Piece::new('P');
		board.grid[1][3] = Piece::new('p');
		board.grid[6][0] = Piece::new('P');
		board.grid[1][0] = Piece::new('p');

		board.move_piece(&Move::from_str("d7d5").unwrap());

		assert!(board.moves().contains(&Move::from_str("e5d6").unwrap()));

		board.move_piece(&Move::from_str("a2a3").unwrap()); //Trekk som ikke gjør noe
		board.move_piece(&Move::from_str("a7a6").unwrap()); //Trekk som ikke gjør noe
		
		//Nå skal det ikke lenger være mulig å ta en passant
		assert!( ! board.moves().contains(&Move::from_str("e5d6").unwrap()));		
	}

	#[test]
	fn google_en_passant(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[6][0] = Piece::new('P');
		board.grid[4][1] = Piece::new('p');
		
		board.move_piece(&Move::from_str("a2a4").unwrap());
		board.move_piece(&Move::from_str("b4a3").unwrap());

		assert_eq!(board.get_reference_at(0, 4), &None);
	}

	#[test]
	fn holy_hell(){
		let mut board = Board::custom(EMPTY, Black);
		board.grid[1][3] = Piece::new('p');
		board.grid[3][4] = Piece::new('P');

		board.move_piece(&Move::from_str("d7d5").unwrap());
		board.move_piece(&Move::from_str("e5d6").unwrap());

		assert_eq!(board.get_reference_at(3, 3), &None);
	}

	#[test]
	fn can_promote_to_queen(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[1][0] = Piece::new('P');
		board.move_piece(&Move::from_str("a7a8Q").unwrap());

		assert_eq!(board.get_reference_at(0, 0), &Piece::new('Q'));
	}

	#[test]
	fn can_promote_to_knight(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[1][0] = Piece::new('P');
		board.move_piece(&Move::from_str("a7a8N").unwrap());

		assert_eq!(board.get_reference_at(0, 0), &Piece::new('N'));
	}

	#[test]
	fn can_undo_promotion(){
		let mut board = Board::custom(EMPTY, White);
		board.grid[1][0] = Piece::new('P');

		board.move_piece(&Move::from_str("a7a8").unwrap());
		board.go_back();

		assert_eq!(board.get_reference_at(0, 0), &None);
		assert_eq!(board.get_reference_at(0, 1), &Piece::new('P'));
	}

}

#[cfg(test)]
mod test_castling{
	use super::*;

	#[test]
	fn inital_castle(){
		let mut board = Board::new();

		let castle = board.castles[board.counter as usize];
		assert!(castle.0 && castle.1 && castle.2 && castle.3);

		let moves = board.moves();
		assert!( ! moves.contains(&Move::from_str("e1g1").unwrap()));
		assert!( ! moves.contains(&Move::from_str("e8g8").unwrap()));

		board.move_piece(&Move::from_str("e2e4").unwrap()); //Gjør et trekk, slik at det er svart sin tur

		assert!( ! moves.contains(&Move::from_str("e1c1").unwrap()));
		assert!( ! moves.contains(&Move::from_str("e1c8").unwrap()));
	}

	#[test]
	fn custom_board_castle(){
		let setup = "\
-nbqkbnr
pppppppp
--------
--------
----P---
--------
PPPP-PPP
RNBQKBN-";
		let board = Board::custom(setup, White);
		let castle = board.castles[board.counter as usize];
		assert!( ! castle.0 && castle.1 && castle.2 && ! castle.3);
	}

	#[test]
	fn can_castle_short_when_not_obstructed(){
		let mut board = Board::new();
		assert_eq!(0, board.castle_moves().len());

		board.grid[7][6] = None;
		assert_eq!(0, board.castle_moves().len());

		board.grid[7][5] = None;	
		assert!(board.moves().contains(&Move::from_str("e1g1").unwrap()));

		board.color_to_move = Black;
		assert_eq!(0, board.castle_moves().len());

		board.grid[0][5] = None;
		board.grid[0][6] = None;
		assert!(board.moves().contains(&Move::from_str("e8g8").unwrap()));
	}

	#[test]
	fn can_castle_long_when_not_obstructed(){
		let mut board = Board::new();
		assert_eq!(0, board.castle_moves().len());

		board.grid[7][1] = None;
		assert_eq!(0, board.castle_moves().len());

		board.grid[7][2] = None;
		assert_eq!(0, board.castle_moves().len());

		board.grid[7][3] = None;
		assert!(board.castle_moves().contains(&Move::from_str("e1c1").unwrap()));

		board.color_to_move = Black;
		assert_eq!(0, board.castle_moves().len());

		board.grid[0][3] = None;
		assert_eq!(0, board.castle_moves().len());

		board.grid[0][2] = None;
		assert_eq!(0, board.castle_moves().len());

		board.grid[0][1] = None;
		assert!(board.castle_moves().contains(&Move::from_str("e8c8").unwrap()));
	}
const SIMPLE: &str = "\
r---k--r
pppppppp
--------
--------
--------
--------
PPPPPPPP
R---K--R";
	#[test]
	fn moving_rook_disallows_castling(){
		let mut board = Board::custom(SIMPLE, White);
		let c = board.castles[board.counter as usize];
		assert!(c.0 && c.1 && c.2 && c.3);

		board.move_piece(&Move::from_str("h1g1").unwrap());
		let c = board.castles[board.counter as usize];
		assert!( ! c.0);

		board.move_piece(&Move::from_str("a8d8").unwrap());
		board.move_piece(&Move::from_str("a1c1").unwrap());

		let c = board.castles[board.counter as usize];
		assert!( !c.0 && !c.1 && c.2 && !c.3);
	}

	#[test]
	fn moving_king_disallows_castling(){
		let mut board = Board::custom(SIMPLE, White);
		board.move_piece(&Move::from_str("e1f1").unwrap());

		let c = board.castles[board.counter as usize];
		assert!(!c.0 && !c.1 && c.2 && c.3);

		board.move_piece(&Move::from_str("e8d8").unwrap());
		let c = board.castles[board.counter as usize];
		assert!(!c.0 && !c.1 && !c.2 && !c.3);
	}

	#[test]
	fn castling_also_moves_the_rook(){
		let mut board = Board::custom(SIMPLE, White);
		println!("1\n{}", board.to_string());
		board.move_piece(&Move::from_str("e1g1").unwrap());
		assert_eq!(board.get_reference_at(4, 7), &None);
		assert_eq!(board.get_reference_at(7, 7), &None);
		assert_eq!(board.get_reference_at(6, 7), &Piece::new('K'));
		assert_eq!(board.get_reference_at(5, 7), &Piece::new('R'));
		println!("2\n{}", board.to_string());
		board.move_piece(&Move::from_str("e8c8").unwrap());
		assert_eq!(board.get_reference_at(0, 0), &None);
		assert_eq!(board.get_reference_at(1, 0), &None);
		assert_eq!(board.get_reference_at(4, 0), &None);
		assert_eq!(board.get_reference_at(3, 0), &Piece::new('r'));
		assert_eq!(board.get_reference_at(2, 0), &Piece::new('k'));
		println!("3\n{}", board.to_string());
		board.go_back();
		println!("3.5");
		board.go_back();
		println!("4\n{}", board.to_string());
		assert_eq!(board.get_reference_at(4, 7), &Piece::new('K'));
		assert_eq!(board.get_reference_at(7, 7), &Piece::new('R'));
		assert_eq!(board.get_reference_at(6, 7), &None);
		assert_eq!(board.get_reference_at(5, 7), &None);
		println!("5\n{}", board.to_string());
		board.move_piece(&Move::from_str("e1c1").unwrap());
		assert_eq!(board.get_reference_at(0, 7), &None);
		assert_eq!(board.get_reference_at(1, 7), &None);
		assert_eq!(board.get_reference_at(2, 7), &Piece::new('K'));
		assert_eq!(board.get_reference_at(3, 7), &Piece::new('R'));
		assert_eq!(board.get_reference_at(4, 7), &None);
		println!("6\n{}", board.to_string());
		board.move_piece(&Move::from_str("e8g8").unwrap());
		assert_eq!(board.get_reference_at(7, 0), &None);
		assert_eq!(board.get_reference_at(4, 0), &None);
		assert_eq!(board.get_reference_at(5, 0), &Piece::new('r'));
		assert_eq!(board.get_reference_at(6, 0), &Piece::new('k'));
	}

	//Denne testen er kun for å finne verdien til de forskjellige rokadene, 
	//siden vi ikke har lov til å finne dem i en const.
	#[test]
	fn find_castle_values(){
	    let K = Piece::new('K').unwrap();
	    let R = Piece::new('R').unwrap();
	    let k = Piece::new('k').unwrap();
	    let r = Piece::new('r').unwrap();
	    let ws = K.value_at(&Position{x: 6, y: 7}) - K.value_at(&Position{x: 4, y: 7}) 
	            + R.value_at(&Position{x: 5, y: 7}) - R.value_at(&Position{x: 7, y: 7});
	    let wl = K.value_at(&Position{x: 2, y: 7}) - K.value_at(&Position{x: 4, y: 7})
	            + R.value_at(&Position{x: 3, y: 7}) - R.value_at(&Position{x: 0, y: 7});
        let bs = k.value_at(&Position{x: 6, y: 0}) - k.value_at(&Position{x: 4, y: 0})
        		+ r.value_at(&Position{x: 5, y: 0}) - r.value_at(&Position{x: 7, y: 0});
		let bl = k.value_at(&Position{x: 2, y: 0}) - k.value_at(&Position{x: 4, y: 0})
				+ r.value_at(&Position{x: 3, y: 0}) - r.value_at(&Position{x: 0, y: 0});
	    println!("White short: {}\nWhite long: {}\nBlack short: {}\nBlack long: {}", ws, wl, bs, bl);
	    //panic!();
	}

	#[test]
	fn cannot_castle_when_in_check(){
		let mut board = Board::custom(SIMPLE, White);
		board.grid[6][4] = Piece::new('r');

		assert!( ! board.moves().contains(&Move::from_str("e1g1").unwrap()));
	}

	#[test]
	fn cannot_castle_through_check(){
		let mut board = Board::custom(SIMPLE, Black);
		board.grid[1][3] = Piece::new('R');

		assert!( ! board.moves().contains(&Move::from_str("e8c8").unwrap()));
	}

	#[test]
	fn cannot_castle_into_check(){
		let mut board = Board::custom(SIMPLE, Black);
		board.grid[6][2] = Piece::new('r');

		assert!( ! board.moves().contains(&Move::from_str("e1c1").unwrap()));
	}
}

#[cfg(test)]
mod test_movement{
	use super::*;

	const e4: &str = "\
rnbqkbnr
pppppppp
--------
--------
----P---
--------
PPPP-PPP
RNBQKBNR";
	const e4c5: &str = "\
rnbqkbnr
pp-ppppp
--------
--p-----
----P---
--------
PPPP-PPP
RNBQKBNR";
	const e4c6: &str = "\
rnbqkbnr
pp-ppppp
--p-----
--------
----P---
--------
PPPP-PPP
RNBQKBNR";	

	#[test]
	fn just_e4(){
		let mut board = Board::new();
		let expected = Board::custom(e4, Black);
		board.move_piece(&Move::from_str("e2e4").unwrap());

		assert_eq!(board.grid, expected.grid);
	}

	#[test]
	fn e4_and_back(){
		let mut board = Board::new();
		board.move_piece(&Move::from_str("e2e4").unwrap());
		board.go_back();
		assert_eq!(board, Board::new());
	}

	#[test]
	fn e4_c5_and_back(){
		let mut board = Board::new();

		board.move_piece(&Move::from_str("e2e4").unwrap());
		assert_eq!(board.grid, Board::custom(e4, Black).grid);

		board.move_piece(&Move::from_str("c7c5").unwrap());
		assert_eq!(board.grid, Board::custom(e4c5, White).grid);

		board.go_back();
		assert_eq!(board.grid, Board::custom(e4, Black).grid);

		board.go_back();
		assert_eq!(board, Board::new());
	}

	#[test]
	fn piece_capture(){
		let e2e7 = "\
rnbqkbnr
ppppPppp
--------
--------
--------
--------
PPPP-PPP
RNBQKBNR";
		
		let mut board = Board::new();

		board.move_piece(&Move::from_str("e2e7").unwrap());
		assert_eq!(board.grid, Board::custom(e2e7, Black).grid);

		board.go_back();
		assert_eq!(board, Board::new());
	}
	#[test]
	#[should_panic]
	fn cannot_go_back_from_inital_state(){
		let mut board = Board::new();
		board.go_back();
	}

	#[test]
	fn e4_c5_back_c6_back_back(){
		let mut board = Board::new();

		board.move_piece(&Move::from_str("e2e4").unwrap());
		assert_eq!(board.grid, Board::custom(e4, Black).grid);

		board.move_piece(&Move::from_str("c7c5").unwrap());
		assert_eq!(board.grid, Board::custom(e4c5, White).grid);

		board.go_back();
		board.move_piece(&Move::from_str("c7c6").unwrap());
		assert_eq!(board.grid, Board::custom(e4c6, White).grid);

		board.go_back();
		board.go_back();
		assert_eq!(board, Board::new());
	}
}

#[cfg(test)]
mod zobrist_testing{
	use super::*;

	#[test]
	fn zobrist_doesnt_crash(){
		let board = Board::new();
		let mut zob = Zobrist::new(&board.grid, White);
		println!("Current hash is: {}", zob.hash());
		//panic!();
	}
	#[test]
	fn moving_piece_changes_hash(){
		let mut board = Board::new();
		let hash1 = board.hash();
		board.move_str("e2e4");
		let hash2 = board.hash();

		assert_ne!(hash1, hash2);
	}

	#[test]
	fn different_sequence_same_result_same_hash(){
		let mut board1 = Board::new();
		let mut board2 = Board::new();

		board1.move_str("e2e4");
		board1.move_str("e7e5");
		board1.move_str("d2d4");
		board1.move_str("b8c6");

		board2.move_str("d2d4");
		board2.move_str("e7e5");
		board2.move_str("e2e4");
		board2.move_str("b8c6");

		println!("{}{}", board1.to_string(), board2.to_string());
		assert_eq!(board1.grid, board2.grid);
		assert_eq!(board1.hash(), board2.hash());
	}

	#[test]
	fn zobrist_handles_castling_rights(){
		let mut board1 = Board::new();
		let mut board2 = Board::new();

		board1.move_str("e2e4");
		board1.move_str("e7e5");
		board1.move_str("e1e2");
		board1.move_str("b8c6");
		board1.move_str("e2e1");

		board2.move_str("e2e4");
		board2.move_str("e7e5");
		board2.move_str("d1e2");
		board2.move_str("b8c6");
		board2.move_str("e2d1");

		assert_eq!(board1.grid, board2.grid); 		//Stillingen ser lik ut, men
		assert_ne!(board1.hash(), board2.hash()) 	//det første brettet kan ikke lenger rokere med hvit
	}
	#[test]
	fn moving_back_gives_same_hash(){
		let mut board = Board::new();
		let hash1 = board.hash();
		board.move_str("b1c3");
		board.move_str("g8f6");

		let hash2 = board.hash();

		board.move_str("c3b1");
		board.move_str("f6g8");

		let hash3 = board.hash();

		assert_ne!(hash1, hash2);
		assert_eq!(hash1, hash3);
	}

	#[test]
	fn zobrist_handles_en_passant(){
		let pp = "\
rnbqkbnr
pppppppp
--------
--------
--------
-------P
PPPPPPP-
RNBQKBNR";
		let mut board1 = Board::new();
		let mut board2 = Board::custom(pp, White);

		assert_ne!(board1.hash(), board2.hash());

		board1.move_str("h2h4");
		board2.move_str("h3h4");

		assert_eq!(board1.grid, board2.grid);
		assert_ne!(board1.hash(), board2.hash());

		board1.move_str("a7a6");
		board2.move_str("a7a6");

		assert_eq!(board1.hash(), board2.hash());
	}

	#[test]
	fn different_moves_same_position(){
		let mut french = Board::new();
		let mut petrov = Board::new();

		french.move_str("e2e4");
		french.move_str("e7e6");
		french.move_str("d2d4");
		french.move_str("d7d5");		
		french.move_str("e4d5");
		french.move_str("e6d5");
		french.move_str("g1f3");
		french.move_str("g8f6");

		petrov.move_str("e2e4");
		petrov.move_str("e7e5");
		petrov.move_str("g1f3");
		petrov.move_str("g8f6");
		petrov.move_str("f3e5");
		petrov.move_str("d7d6");
		petrov.move_str("e5f3");
		petrov.move_str("f6e4");
		petrov.move_str("d2d3");
		petrov.move_str("e4f6");
		petrov.move_str("d3d4");
		petrov.move_str("d6d5");

		assert_ne!(french, petrov);
		assert_eq!(french.hash(), petrov.hash());

	}
}