pub use crate::piece::{Piece, Color};
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
	counter: u16,
	graveyard: Vec<TombStone>,
	moves: Vec<Move>

	//TODO blir mange flere felt her etter hvert.
}

#[derive(Copy, Clone, PartialEq)]
pub struct Move{
	from: Position,
	to: Position,
	value: Option<Score>
}

#[derive(Copy, Clone, PartialEq)]
pub struct Score(u32);

#[derive(Copy, Clone, PartialEq)]
struct Position{
	x: usize,
	y: usize
}

//Holder styr på hvor og når brikker blir tatt, så de kan respawnes etterpå.
#[derive(PartialEq)]
struct TombStone{
	piece: Piece,
	date: u16,
	position: Position
}

impl Board{
	pub fn new() -> Self{
		Board::custom_board(DEFAULT_BOARD, Color::White)
	}

	pub fn move_piece(&mut self, m: Move){
		let pie = self.get_piece_at(&m.from);
		if let Some(target) = self.get_piece_at(&m.to){
			self.graveyard.push(TombStone{piece: target, position: m.to, date: self.counter})
		}
		self.grid[m.from.y][m.from.x] = None;
		self.grid[m.to.y][m.to.x] = pie;

		self.moves.push(m);
		self.counter += 1;
		self.color_to_move = self.color_to_move.opposite();
	}

	pub fn go_back(&mut self){
		let m = self.moves.pop().expect("Cannot go further back!");

		self.counter -= 1;
		self.color_to_move = self.color_to_move.opposite();

		let pie = self.get_piece_at(&m.to);
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

	fn get_piece_at(&self, p: &Position) -> Option<Piece>{
		self.grid[p.y][p.x].clone()
	}

	fn custom_board(s: &str, c: Color) -> Self{
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
		Board{grid, color_to_move: c, score: Score(0), counter: 0, graveyard: Vec::new(), moves: Vec::new()}
	}


}

impl Move{
	//Lager et nytt move. NB! Denne bryr seg kun om koordinater, der Origo er oppe til venstre.
	//Dermed er e1->e2 det samme som (4, 7, 4, 6).
	pub fn new(filefrom: usize, rankfrom: usize, fileto: usize, rankto: usize) -> Self{
		Move{from: Position{x: filefrom, y: rankfrom}, to: Position{x: fileto, y: rankto}, value: None}
	}

	pub fn value(&self) -> Score{
		match self.value{
			None    => { panic!("This move has no associated value."); }
			Some(v) => v
		}
	}
}

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
		write!(f, "grid:\n{}counter: {}\ngraveyard: {:?}\n, color: {}", self.to_string(), self.counter, self.graveyard, self.color_to_move)
	}
}

impl fmt::Debug for TombStone{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "turn: {}, piece: {}", self.date, self.piece) //TODOOOO
	}
}

#[cfg(test)]
mod test_movement{
	use super::*;
	#[test]
	fn e2e4(){
		let e4 = "\
rnbqkbnr
pppppppp
--------
--------
----P---
--------
PPPP-PPP
RNBQKBNR";
		let mut board = Board::new();
		let expected = Board::custom_board(e4, Color::Black);
		board.move_piece(Move::new(4, 6, 4, 4));

		assert_eq!(board.grid, expected.grid);
	}

	#[test]
	fn e2e4_and_back(){
		let mut board = Board::new();
		board.move_piece(Move::new(4, 6, 4, 4));
		board.go_back();
		assert_eq!(board, Board::new());
	}

	#[test]
	fn e2e4_c7c5_and_back(){
		let e4 = "\
rnbqkbnr
pppppppp
--------
--------
----P---
--------
PPPP-PPP
RNBQKBNR";
		let c5 = "\
rnbqkbnr
pp-ppppp
--------
--p-----
----P---
--------
PPPP-PPP
RNBQKBNR";
		let mut board = Board::new();

		board.move_piece(Move::new(4, 6, 4, 4));
		assert_eq!(board.grid, Board::custom_board(e4, Color::Black).grid);

		board.move_piece(Move::new(2, 1, 2, 3));
		assert_eq!(board.grid, Board::custom_board(c5, Color::White).grid);

		board.go_back();
		assert_eq!(board.grid, Board::custom_board(e4, Color::Black).grid);

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

		board.move_piece(Move::new(4, 6, 4, 1));
		assert_eq!(board.grid, Board::custom_board(e2e7, Color::Black).grid);

		board.go_back();
		assert_eq!(board, Board::new());
	}
	#[test]
	#[should_panic]
	fn cannot_go_back_from_inital_state(){
		let mut board = Board::new();
		board.go_back();
	}
}