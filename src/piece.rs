use std::fmt;

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

impl Piece{
	pub fn new(c: char) -> Option<Piece>{
		let color = if c.is_uppercase() { Color::White } else { Color::Black };
		match c.to_lowercase().next().unwrap(){
			'p' => Some(Piece{piecetype: PieceType::Pawn, color}),
			'r' => Some(Piece{piecetype: PieceType::Rook, color}),
			'n' => Some(Piece{piecetype: PieceType::Knight, color}),
			'b' => Some(Piece{piecetype: PieceType::Bishop, color}),
			'q' => Some(Piece{piecetype: PieceType::Queen, color}),
			'k' => Some(Piece{piecetype: PieceType::King, color}),
			'-' => None,
			_   => { panic!("Got an unexpected character when creating a piece: {}", c); }
		}
	}

	pub fn char(&self) -> char{
		let c = match self.piecetype{
			PieceType::Pawn => 'p',
			PieceType::King => 'k',
			PieceType::Queen => 'q',
			PieceType::Bishop => 'b',
			PieceType::Knight => 'n',
			PieceType::Rook => 'r',
		};
		if self.color == Color::White{
			c.to_uppercase().next().unwrap()
		} else { c }
	}
}

impl Color {
	pub fn opposite(&self) -> Color{
		match self{
			Color::White => Color::Black,
			Color::Black => Color::White
		}
	}
}

impl fmt::Display for Color{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let s = match self{
			Color::White => "White",
			Color::Black => "Black"
		};
		write!(f, "{}", s)
	}
}

impl fmt::Display for PieceType{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let s = match self{
			PieceType::King => "King",
			PieceType::Queen => "Queen",
			PieceType::Bishop => "Bishop",
			PieceType::Knight => "Knight",
			PieceType::Rook => "Rook",
			PieceType::Pawn => "Pawn"
		};
		write!(f, "{}", s)
	}
}

impl fmt::Display for Piece{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} {}", self.piecetype, self.color)
	}
}