pub use crate::piece::{Piece, Color};
use asciis::asc::Asciis;

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

pub struct Board{
	grid: [[Option<Piece>; 8]; 8],
	color_to_move: Color,
	score: Score
	//TODO blir mange flere felt her etter hvert.
}

pub struct Move{
	from: Position,
	to: Position,
	value: Option<Score>
}

pub struct Score(u32);

struct Position{
	y: usize,
	x: usize
}

impl Board{
	pub fn new() -> Self{
		Board::custom_board(DEFAULT_BOARD, Color::White)
	}

	pub fn move_piece(&mut self, m: Move){
		let pie = self.get_piece_at(&m.from);
		self.grid[m.from.y][m.from.x] = None;
		self.grid[m.to.y][m.to.x] = pie;
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
		Board{grid, color_to_move: c, score: Score(0)}
	}


}

impl Move{
	//pub fn new(filefrom: char, rankfrom: usize, fileto: char, rankto: usize){
		//let asc = Asciis{};
		//let filefromint = asc.ord(filefrom.to_lowercase().next().unwrap());
		//println!("{}", filefromint);
	//}
	pub fn new(filefrom: usize, rankfrom: usize, fileto: usize, rankto: usize) -> Self{
		Move{from: Position{x: filefrom, y: rankfrom}, to: Position{x: fileto, y: rankto}, value: None}
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
