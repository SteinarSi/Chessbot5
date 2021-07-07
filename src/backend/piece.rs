use std::fmt;
use PieceType::*;
use Color::*;
use super::movement::{Score, Position};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color{
	White,
	Black
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceType{
	King,
	Queen, 
	Bishop,
	Knight, 
	Rook, 
	Pawn
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Piece{
	pub piecetype: PieceType,
	pub color: Color
}

pub type Vector = (i8, i8);

impl Piece{
	pub fn new(c: char) -> Option<Piece>{
		let color = if c.is_uppercase() { White } else { Black };
		match c.to_lowercase().next().unwrap(){
			'p' => Some(Piece{piecetype: Pawn, color}),
			'r' => Some(Piece{piecetype: Rook, color}),
			'n' => Some(Piece{piecetype: Knight, color}),
			'b' => Some(Piece{piecetype: Bishop, color}),
			'q' => Some(Piece{piecetype: Queen, color}),
			'k' => Some(Piece{piecetype: King, color}),
			'-' => None,
			_   => { panic!("Got an unexpected character when creating a piece: {}", c); }
		}
	}

	pub fn directions(&self) -> &[Vector]{
		match self.piecetype{
			King   => &[(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)],
			Queen  => &[(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)],
			Bishop => &[(-1, -1), (1, -1), (1, 1), (-1, 1)],
			Knight => &[(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)],
			Rook   => &[(0, -1), (1, 0), (0, 1), (-1, 0)],
			Pawn   => if self.color == White { &[(-1, -1), (0, -1), (1, -1)] } else {&[(-1,  1), (0,  1), (1,  1)]}
		}
	}

	pub fn can_run(&self) -> bool{
		match self.piecetype{
			Rook   => true,
			Bishop => true,
			Queen  => true,
			_      => false
		}
	}

	pub fn char(&self) -> char{
		let c = match self.piecetype{
			Pawn   => 'p',
			King   => 'k',
			Queen  => 'q',
			Bishop => 'b',
			Knight => 'n',
			Rook   => 'r',
		};
		if self.color == White{
			c.to_uppercase().next().unwrap()
		} else { c }
	}

	//Den posisjonelle verdien til en brikke. Legg merke til at svarts verdi er negert.
	pub fn value_at(&self, p: &Position) -> Score{
		match self.piecetype{
			King   => if self.color == White { MG_KING_TABLE[p.y][p.x] }   else { - MG_KING_TABLE[7 - p.y][p.x] }
			Queen  => if self.color == White { MG_QUEEN_TABLE[p.y][p.x] }  else { - MG_QUEEN_TABLE[7 - p.y][p.x] }
			Bishop => if self.color == White { MG_BISHOP_TABLE[p.y][p.x] } else { - MG_BISHOP_TABLE[7 - p.y][p.x] }
			Knight => if self.color == White { MG_KNIGHT_TABLE[p.y][p.x] } else { - MG_KNIGHT_TABLE[7 - p.y][p.x] }
			Rook   => if self.color == White { MG_ROOK_TABLE[p.y][p.x] }   else { - MG_ROOK_TABLE[7 - p.y][p.x] }
			Pawn   => if self.color == White { MG_PAWN_TABLE[p.y][p.x] }   else { - MG_PAWN_TABLE[7 - p.y][p.x] }			
		}
	}

	//Based on PeSTO's evaluation function: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
	pub fn inherent_value(&self) -> Score{
		let s = match self.piecetype{
			Pawn => 82,
			Knight => 337,
			Bishop => 365,
			Rook => 477,
			Queen => 1025,
			King => 0
		};
		if self.color == White { s } else { -s }
	}

	pub fn combined_value_at(&self, pos: &Position) -> Score{
		self.inherent_value() + self.value_at(pos)
	}

	pub fn type_index(&self) -> usize{
		match (self.piecetype, self.color){
			(Pawn,   White) => 0,
			(Pawn,   Black) => 1,
			(Rook,   White) => 2,
			(Rook,   Black) => 3,
			(Knight, White) => 4,
			(Knight, Black) => 5,
			(Bishop, White) => 6,
			(Bishop, Black) => 7,
			(Queen,  White) => 8,
			(Queen,  Black) => 9,
			(King,   White) => 10,
			(King,   Black) => 11
		}
	}

}

impl Color {
	pub fn opposite(&self) -> Color{
		match self{
			White => Black,
			Black => White
		}
	}
}

impl fmt::Display for Color{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let s = match self{
			White => "White",
			Black => "Black"
		};
		write!(f, "{}", s)
	}
}

impl fmt::Display for PieceType{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let s = match self{
			King   => "King",
			Queen  => "Queen",
			Bishop => "Bishop",
			Knight => "Knight",
			Rook   => "Rook",
			Pawn   => "Pawn"
		};
		write!(f, "{}", s)
	}
}

impl fmt::Display for Piece{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} {}", self.piecetype, self.color)
	}
}


///////////////////////////////////////
//        Piece square values        //
///////////////////////////////////////

//These values are all based on PeSTO's evaluation function: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
const MG_PAWN_TABLE: [[Score; 8]; 8] = [
		[ 0,   0,   0,   0,   0,   0,  0,   0],
	    [ 98, 134,  61,  95,  68, 126, 34, -11],
	    [ -6,   7,  26,  31,  65,  56, 25, -20],
	    [-14,  13,   6,  21,  23,  12, 17, -23],
	    [-27,  -2,  -5,  12,  17,   6, 10, -25],
	    [-26,  -4,  -4, -10,   3,   3, 33, -12],
	    [-35,  -1, -20, -23, -15,  24, 38, -22],
	    [  0,   0,   0,   0,   0,   0,  0,   0]
	];

const MG_KNIGHT_TABLE: [[Score; 8]; 8] = [
    	[-167, -89, -34, -49,  61, -97, -15, -107],
    	[ -73, -41,  72,  36,  23,  62,   7,  -17],
    	[ -47,  60,  37,  65,  84, 129,  73,   44],
    	[  -9,  17,  19,  53,  37,  69,  18,   22],
    	[ -13,   4,  16,  13,  28,  19,  21,   -8],
    	[ -23,  -9,  12,  10,  19,  17,  25,  -16],
    	[ -29, -53, -12,  -3,  -1,  18, -14,  -19],
    	[-105, -21, -58, -33, -17, -28, -19,  -23]
	];

const MG_BISHOP_TABLE: [[Score; 8]; 8] = [
	    [-29,   4, -82, -37, -25, -42,   7,  -8],
	    [-26,  16, -18, -13,  30,  59,  18, -47],
	    [-16,  37,  43,  40,  35,  50,  37,  -2],
	    [ -4,   5,  19,  50,  37,  37,   7,  -2],
	    [ -6,  13,  13,  26,  34,  12,  10,   4],
	    [  0,  15,  15,  15,  14,  27,  18,  10],
	    [  4,  15,  16,   0,   7,  21,  33,   1],
	    [-33,  -3, -14, -21, -13, -12, -39, -21]
	];

const MG_ROOK_TABLE: [[Score; 8]; 8] = [
	    [ 32,  42,  32,  51, 63,  9,  31,  43],
	    [ 27,  32,  58,  62, 80, 67,  26,  44],
	    [ -5,  19,  26,  36, 17, 45,  61,  16],
	    [-24, -11,   7,  26, 24, 35,  -8, -20],
	    [-36, -26, -12,  -1,  9, -7,   6, -23],
	    [-45, -25, -16, -17,  3,  0,  -5, -33],
	    [-44, -16, -20,  -9, -1, 11,  -6, -71],
	    [-19, -13,   1,  17, 16,  7, -37, -26]
	];

const MG_QUEEN_TABLE: [[Score; 8]; 8] = [
		[-28,   0,  29,  12,  59,  44,  43,  45],
	    [-24, -39,  -5,   1, -16,  57,  28,  54],
	    [-13, -17,   7,   8,  29,  56,  47,  57],
	    [-27, -27, -16, -16,  -1,  17,  -2,   1],
	    [ -9, -26,  -9, -10,  -2,  -4,   3,  -3],
	    [-14,   2, -11,  -2,  -5,   2,  14,   5],
	    [-35,  -8,  11,   2,   8,  15,  -3,   1],
	    [ -1, -18,  -9,  10, -15, -25, -31, -50]
	];

const MG_KING_TABLE: [[Score; 8]; 8] = [
	    [-65,  23,  16, -15, -56, -34,   2,  13],
	    [ 29,  -1, -20,  -7,  -8,  -4, -38, -29],
	    [ -9,  24,   2, -16, -20,   6,  22, -22],
	    [-17, -20, -12, -27, -30, -25, -14, -36],
	    [-49,  -1, -27, -39, -46, -44, -33, -51],
	    [-14, -14, -22, -46, -44, -30, -15, -27],
	    [  1,   7,  -8, -64, -43, -16,   9,   8],
	    [-15,  36,  12, -54,   8, -28,  24,  14]
	];



#[cfg(test)]
mod piece_value_tests{
	use super::*;
	#[test]
	fn zero_sum_piece_values(){
		let K = Piece::new('K').unwrap();
		let k = Piece::new('k').unwrap();

		assert_eq!(0, K.combined_value_at(&Position{x: 4, y: 7}) + k.combined_value_at(&Position{x: 4, y: 0}));

		let Q = Piece::new('Q').unwrap();
		let q = Piece::new('q').unwrap();

		assert_eq!(0, Q.combined_value_at(&Position{x: 6, y: 3}) + q.combined_value_at(&Position{x: 6, y: 4}));

		let P = Piece::new('P').unwrap();
		let p = Piece::new('p').unwrap();

		assert_eq!(0, P.combined_value_at(&Position{x: 2, y: 1}) + p.combined_value_at(&Position{x: 2, y: 6}));
	}
}