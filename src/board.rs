pub use crate::piece::{Piece, Color};

const BOARD_SIZE: usize = 8;
const DEFAULT_BOARD: &str = "\
rnbqkbnr
pppppppp
________
________
________
________
PPPPPPPP
RNBQKBNR";

pub struct Board{
	grid: [[Option<Piece>; 8]; 8],
	color_to_move: Color
	//TODO blir mange flere felt her etter hvert.
}

impl Board{
	pub fn new() -> Self{
		Board::custom_board(DEFAULT_BOARD.to_string(), Color::White)
	}

	fn custom_board(s: String, c: Color) -> Self{
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
		Board{grid, color_to_move: c}
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
				else { ret.push('_'); }
				
			}
			ret.push('\n');
		}
		ret
	}	
}
