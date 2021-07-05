//Denne filen skulle gjerne hett move.rs, men move er allerede et nøkkelord i Rust og kan ikke brukes :(((
use std::fmt;
use std::char;
use super::piece::Piece;

#[derive(Copy, Clone)]
pub struct Move{
	pub from: Position,
	pub to: Position,
	heurestic_value: Score,
	actual_value: Option<Score>,
	pub promote: Option<Piece>
}

pub type Score = i32;

#[derive(Copy, Clone, PartialEq)]
pub struct Position{
	pub x: usize,
	pub y: usize
}

impl Move{
	//Lager et nytt move. NB! Denne bryr seg kun om koordinater, der Origo er oppe til venstre.
	//Dermed er e1->e2 det samme som (4, 7, 4, 6).
	pub fn new(filefrom: usize, rankfrom: usize, fileto: usize, rankto: usize, promote: Option<Piece>, heurestic_value: Score) -> Self{
		Move{from: Position{x: filefrom, y: rankfrom}, to: Position{x: fileto, y: rankto}, actual_value: None, promote, heurestic_value}
	}

	//Parser en streng på formen "e2e4". "e4", "e2-e4", "Pe4" er ikke gyldig og gir None.
	pub fn from_str(s: &str) -> Option<Move>{
		let mut l = s.chars();
		let filefrom = (l.next()? as u32 - 97) as usize;
		let rankfrom = (56 - l.next()? as u32) as usize;
		let fileto   = (l.next()? as u32 - 97) as usize;
		let rankto   = (56 - l.next()? as u32) as usize;
		match l.next(){
			None => Some(Move::new(filefrom, rankfrom, fileto, rankto, None, 0)),
			Some(c) => Some(Move::new(filefrom, rankfrom, fileto, rankto, Piece::new(c), 0))
		}
		
	}

	pub fn actual_value(&self) -> Score{
		match self.actual_value{
			None    => { panic!("This move has no associated value."); }
			Some(v) => v
		}
	}

	pub fn heurestic_value(&self) -> Score{
		self.heurestic_value
	}

	pub fn set_actual_value(&mut self, s: Score){
		self.actual_value = Some(s);
	}
}

impl PartialEq for Move{
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.promote == other.promote
    }
}

impl ToString for Move{
	fn to_string(&self) -> String{
		let mut ret = String::new();
		ret.push('(');
		ret.push(char::from_u32(self.from.x as u32 + 97).unwrap());
		ret.push(char::from_digit((8 - self.from.y) as u32, 10).unwrap());
		ret.push_str(", ");
		ret.push(char::from_u32(self.to.x as u32 + 97).unwrap());
		ret.push(char::from_digit((8 - self.to.y) as u32, 10).unwrap());
		ret.push(')');
		ret
	}
}

impl fmt::Debug for Move{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_string())
	}
}