pub use super::piece::{Piece, PieceType, Color, Color::White, Color::Black};
pub use super::movement::{Move, Score};
use super::movement::Position;
use std::fmt;

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

#[derive(PartialEq)]
pub struct Board{
	grid: [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE],
	color_to_move: Color,
	score: Score,
	counter: usize,
	graveyard: Vec<TombStone>,
	moves: Vec<Move>,
	passants: Vec<Option<i8>>,
	castles: Vec<Castle>

	//TODO blir mange flere felt her etter hvert.
}

//Holder styr på hvor og når brikker blir tatt, så de kan respawnes etterpå.
#[derive(PartialEq)]
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
			self.graveyard.push(TombStone{piece: target, position: m.to, date: self.counter})
		}
		self.grid[m.from.y][m.from.x] = None;
		self.grid[m.to.y][m.to.x] = Some(pie);

		let mut passant = None;
		if pie.piecetype == PieceType::Pawn{
			if (m.from.y as i8 - m.to.y as i8).abs() == 2{
				passant = Some(m.from.x as i8);
			}
			if let Some(ps) = self.passants[self.counter]{
				if m.to.x == ps as usize {
					if m.to.y == 2 {
						let pos = Position{x: m.to.x, y: 3};
						let target = self.get_clone_at(&pos).unwrap();
						self.graveyard.push(TombStone{piece: target, position: pos, date: self.counter});
						self.grid[3][m.to.x] = None;
					}
					if m.to.y == 5 {
						let pos = Position{x: m.to.x, y: 4};
						let target = self.get_clone_at(&pos).unwrap();
						self.graveyard.push(TombStone{piece: target, position: pos, date: self.counter});
						self.grid[4][m.to.x] = None;
					}
				}
			}
		}
		if pie.piecetype == PieceType::King{
			self.move_rook_if_castling(m);
		}
		self.update_castle(m);
		self.passants.push(passant);
		self.moves.push(*m);
		self.counter += 1;
		self.color_to_move = self.color_to_move.opposite();
	}

	pub fn go_back(&mut self){
		let m = self.moves.pop().expect("Cannot go further back!");

		self.castles.pop();
		self.counter -= 1;
		self.color_to_move = self.color_to_move.opposite();
		self.passants.pop();

		let pie = self.get_clone_at(&m.to);
		if pie.unwrap().piecetype == PieceType::King{
			self.move_rook_if_castling(&m);
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
	//TODO: Denne tar ennå ikke hensyn til promotering, eller om trekket setter kongen i sjakk.
	pub fn moves(&self) -> Vec<Move>{
		let mut ret = Vec::new();
		let color = self.color_to_move;

		for y in 0..BOARD_SIZE{
			for x in 0..BOARD_SIZE{
				if let Some(p) = self.get_reference_at(x, y){
					if p.color == color{
						if p.piecetype == PieceType::Pawn{
							ret.append(&mut self.pawn_moves(x, y, p));
						}
						else if p.can_run(){
							ret.append(&mut self.running_moves(x, y, p));
						}
						else{
							ret.append(&mut self.walking_moves(x, y, p));
						}
					}
				}
			}
		}
		ret.append(&mut self.castle_moves());
		ret
	}

	fn walking_moves(&self, x: usize, y: usize, p: &Piece) -> Vec<Move>{
		let mut ret = Vec::new();
		let color = self.color_to_move;
		for dir in p.directions(){
			let to_x = x as i8 + dir.0;
			let to_y = y as i8 + dir.1;
			if to_x < 0 || to_x > 7 || to_y < 0 || to_y > 7 { continue; }
			if let &Some(t) = self.get_reference_at(to_x as usize, to_y as usize) {
				if t.color == color { continue; }
			}
			ret.push(Move::new(x, y, to_x as usize, to_y as usize));
		}
		ret
	}

	//Trekkene til alle brikker som kan gå flere skritt om gangen.
	fn running_moves(&self, x: usize, y: usize, p: &Piece) -> Vec<Move>{
		let mut ret = Vec::new();
		let color = self.color_to_move;
		for dir in p.directions(){
			let mut to_x = x as i8 + dir.0;
			let mut to_y = y as i8 + dir.1;
			loop{
				if to_x < 0 || to_x > 7 || to_y < 0 || to_y > 7 { break; }
				if let &Some(t) = self.get_reference_at(to_x as usize, to_y as usize) {
					if t.color != color { ret.push(Move::new(x, y, to_x as usize, to_y as usize)); }
					break;
				}
				ret.push(Move::new(x, y, to_x as usize, to_y as usize));
				to_x += dir.0;
				to_y += dir.1;
			}
		}
		ret
	}

	//Alle bondetrekk
	fn pawn_moves(&self, x: usize, y: usize, p: &Piece) -> Vec<Move>{
		let mut ret = Vec::new();
		for dir in p.directions(){
			let to_y = y as i8 + dir.1;
			if dir.0 == 0 {
				if to_y < 8 && to_y >= 0 && self.get_reference_at(x, to_y as usize) == &None{
					ret.push(Move::new(x, y, x, to_y as usize));
					if y == 6 && self.color_to_move == White && self.get_reference_at(x, (to_y-1) as usize) == &None{
						ret.push(Move::new(x, y, x, (to_y - 1) as usize));
					} else if y == 1 && self.color_to_move == Black && self.get_reference_at(x, (to_y+1) as usize) == &None{
						ret.push(Move::new(x, y, x, (to_y + 1) as usize));
					}
				}
			}
			else{
				let to_x = x as i8 + dir.0;
				if to_x >= 0 && to_x < 8{
					if let &Some(t) = self.get_reference_at(to_x as usize, to_y as usize) {
						if t.color != self.color_to_move { ret.push(Move::new(x, y, to_x as usize, to_y as usize)); }
					} else if let Some(c) = self.passants[self.counter]{
						if c == to_x { ret.push(Move::new(x, y, to_x as usize, to_y as usize)); }
					}
				}
			}
		}
		ret
	}

	fn move_rook_if_castling(&mut self, m: &Move){
		match (m.from.x, m.from.y, m.to.x, m.to.y) {
			(4, 7, 6, 7) => { self.grid[7].swap(7, 5); },
			(4, 7, 2, 7) => { self.grid[7].swap(0, 3); },
			(4, 0, 6, 0) => { self.grid[0].swap(7, 5); },
			(4, 0, 2, 0) => { self.grid[0].swap(0, 3); },
			_ => {}
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
		self.castles.push(next);
	}

	fn castle_moves(&self) -> Vec<Move>{
		let mut ret = Vec::new();
		let castle = self.castles[self.counter as usize];
		if self.color_to_move == White{
			if castle.0 && self.get_reference_at(5, 7).is_none() && self.get_reference_at(6, 7).is_none(){
				ret.push(Move::new(4, 7, 6, 7));
			}
			if castle.1 && self.get_reference_at(1, 7).is_none() && self.get_reference_at(2, 7).is_none() && self.get_reference_at(3, 7).is_none(){
				ret.push(Move::new(4, 7, 2, 7));
			}
		}
		else{
			if castle.2 && self.get_reference_at(5, 0).is_none() && self.get_reference_at(6, 0).is_none(){
				ret.push(Move::new(4, 0, 6, 0));
			}
			if castle.3 && self.get_reference_at(1, 0).is_none() && self.get_reference_at(2, 0).is_none() && self.get_reference_at(3, 0).is_none(){
				ret.push(Move::new(4, 0, 2, 0));
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

	fn custom(s: &str, c: Color) -> Self{
		let s = s.replace(&['\n'][..], "");
		let mut grid = [[None; BOARD_SIZE]; BOARD_SIZE];
		let mut y = 0;
		let mut x = 0;
		for c in s.chars(){
			grid[y][x] = Piece::new(c);
			x += 1;
			if x == 8{
				y += 1;
				x = 0;
			}
		}
		Board{grid, color_to_move: c, score: 0, counter: 0, graveyard: Vec::new(), moves: Vec::new(), passants: vec![None], castles: vec![Board::build_castle(&grid)]}
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

impl fmt::Debug for Board{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "grid:\n{}counter: {}\ngraveyard: {:?}\n, color: {}\n passants: {:?}", self.to_string(), self.counter, self.graveyard, self.color_to_move, self.passants)
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
#[cfg(test)]
mod test_move_generation{
	use super::*;

	const empty: &str = "\
--------
--------
--------
--------
--------
--------
--------
--------";
	#[test]
	fn just_a_king(){
		let mut board = Board::custom(empty, White);
		board.grid[7][4] = Piece::new('K');
		let moves = board.moves();
		let expected: Vec<Move> = ["e1d2", "e1e2", "e1f2", "e1f1", "e1d1"].iter().map(|s| Move::from_str(s).unwrap()).collect();

		assert_eq!(moves, expected);
	}

	#[test]
	fn just_a_knight(){
		let mut board = Board::custom(empty, White);
		board.grid[0][1] = Piece::new('N');
		let actual = board.moves();
		let expected: Vec<Move> = ["b8c6", "b8d7", "b8a6"].iter().map(|s| Move::from_str(s).unwrap()).collect();

		assert_eq!(expected, actual);
	}

	#[test]
	fn just_a_bishop(){
		let mut board = Board::custom(empty, White);
		board.grid[1][1] = Piece::new('B');
		let actual = board.moves();
		let expected: Vec<Move> = ["b7a8", "b7c8", "b7c6", "b7d5", "b7e4", "b7f3", "b7g2", "b7h1", "b7a6"].iter().map(|s| Move::from_str(s).unwrap()).collect();

		assert_eq!(expected, actual);
	}

	#[test]
	fn can_capture_enemy(){
		let mut board = Board::custom(empty, White);
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
		let mut board = Board::custom(empty, Black);
		board.grid[0][4] = Piece::new('n');
		board.grid[0][3] = Piece::new('q');
		let moves = board.moves();

		assert!( ! moves.contains(&Move::from_str("d8e8").unwrap())); //Kan ikke ta egen brikke
		assert!( ! moves.contains(&Move::from_str("d8f8").unwrap())); //Kan ikke hoppe over egen brikke
		assert!(moves.contains(&Move::from_str("d8c8").unwrap()));   //Kan derimot gjøre vanlige trekk.
	}

	#[test]
	fn just_a_pawn(){
		let mut board = Board::custom(empty, White);
		board.grid[4][3] = Piece::new('P');
		let actual = board.moves();
		let expected = vec![Move::from_str("d4d5").unwrap()];
		assert_eq!(expected, actual);
	}

	#[test]
	fn can_go_two_squares(){
		let mut board = Board::custom(empty, White);
		board.grid[6][0] = Piece::new('P');
		let actual = board.moves();
		let expected = vec![Move::from_str("a2a3").unwrap(), Move::from_str("a2a4").unwrap()];

		assert_eq!(expected, actual);
	}

	#[test]
	fn black_pawns_go_backwards(){
		let mut board = Board::custom(empty, Black);
		board.grid[1][4] = Piece::new('p');
		let actual = board.moves();
		let expected = vec![Move::from_str("e7e6").unwrap(), Move::from_str("e7e5").unwrap()];

		assert_eq!(expected, actual);
	}

	#[test]
	fn pawns_capture_diagonally(){
		let mut board = Board::custom(empty, White);
		board.grid[6][1] = Piece::new('P');
		board.grid[5][0] = Piece::new('p'); //Fiendtlig bonde i rekkevidde
		board.grid[5][2] = Piece::new('P'); //Vennlig bonde i rekkevidde, kan ikke ta denne

		let actual = board.moves();
		let expected: Vec<Move> = vec!["c3c4", "b2a3", "b2b3", "b2b4"].iter().map(|s| Move::from_str(s).unwrap()).collect();

		assert_eq!(expected, actual);
	}

	#[test]
	fn en_passant(){
		let mut board = Board::custom(empty, White);
		board.grid[6][7] = Piece::new('P');
		board.grid[4][6] = Piece::new('p');

		board.move_piece(&Move::from_str("h2h4").unwrap());

		let actual = board.moves();
		let expected = vec![Move::from_str("g4g3").unwrap(), Move::from_str("g4h3").unwrap()];

		assert_eq!(expected, actual);
	}

	#[test]
	fn en_croissant(){
		let mut board = Board::custom(empty, Black);
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
		let mut board = Board::custom(empty, White);
		board.grid[6][0] = Piece::new('P');
		board.grid[4][1] = Piece::new('p');
		
		board.move_piece(&Move::from_str("a2a4").unwrap());
		board.move_piece(&Move::from_str("b4a3").unwrap());

		assert_eq!(board.get_reference_at(0, 4), &None);
	}

	#[test]
	fn holy_hell(){
		let mut board = Board::custom(empty, Black);
		board.grid[1][3] = Piece::new('p');
		board.grid[3][4] = Piece::new('P');

		board.move_piece(&Move::from_str("d7d5").unwrap());
		board.move_piece(&Move::from_str("e5d6").unwrap());

		assert_eq!(board.get_reference_at(3, 3), &None);
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

		board.move_piece(&Move::from_str("e1g1").unwrap());
		assert_eq!(board.get_reference_at(4, 7), &None);
		assert_eq!(board.get_reference_at(7, 7), &None);
		assert_eq!(board.get_reference_at(6, 7), &Piece::new('K'));
		assert_eq!(board.get_reference_at(5, 7), &Piece::new('R'));

		board.move_piece(&Move::from_str("e8c8").unwrap());
		assert_eq!(board.get_reference_at(0, 0), &None);
		assert_eq!(board.get_reference_at(1, 0), &None);
		assert_eq!(board.get_reference_at(4, 0), &None);
		assert_eq!(board.get_reference_at(3, 0), &Piece::new('r'));
		assert_eq!(board.get_reference_at(2, 0), &Piece::new('k'));

		board.go_back();
		board.go_back();
		assert_eq!(board.get_reference_at(4, 7), &Piece::new('K'));
		assert_eq!(board.get_reference_at(7, 7), &Piece::new('R'));
		assert_eq!(board.get_reference_at(6, 7), &None);
		assert_eq!(board.get_reference_at(5, 7), &None);

		board.move_piece(&Move::from_str("e1c1").unwrap());
		assert_eq!(board.get_reference_at(0, 7), &None);
		assert_eq!(board.get_reference_at(1, 7), &None);
		assert_eq!(board.get_reference_at(2, 7), &Piece::new('K'));
		assert_eq!(board.get_reference_at(3, 7), &Piece::new('R'));
		assert_eq!(board.get_reference_at(4, 7), &None);

		board.move_piece(&Move::from_str("e8g8").unwrap());
		assert_eq!(board.get_reference_at(7, 0), &None);
		assert_eq!(board.get_reference_at(4, 0), &None);
		assert_eq!(board.get_reference_at(5, 0), &Piece::new('r'));
		assert_eq!(board.get_reference_at(6, 0), &Piece::new('k'));
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
		board.move_piece(&Move::new(4, 6, 4, 4));

		assert_eq!(board.grid, expected.grid);
	}

	#[test]
	fn e4_and_back(){
		let mut board = Board::new();
		board.move_piece(&Move::new(4, 6, 4, 4));
		board.go_back();
		assert_eq!(board, Board::new());
	}

	#[test]
	fn e4_c5_and_back(){
		let mut board = Board::new();

		board.move_piece(&Move::new(4, 6, 4, 4));
		assert_eq!(board.grid, Board::custom(e4, Black).grid);

		board.move_piece(&Move::new(2, 1, 2, 3));
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

		board.move_piece(&Move::new(4, 6, 4, 1));
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

		board.move_piece(&Move::new(4, 6, 4, 4));
		assert_eq!(board.grid, Board::custom(e4, Black).grid);

		board.move_piece(&Move::new(2, 1, 2, 3));
		assert_eq!(board.grid, Board::custom(e4c5, White).grid);

		board.go_back();
		board.move_piece(&Move::new(2, 1, 2, 2));
		assert_eq!(board.grid, Board::custom(e4c6, White).grid);
	}
}