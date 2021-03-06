//Denne filen skulle gjerne hett move.rs, men move er allerede et nøkkelord i Rust og kan ikke brukes :(((
use std::fmt;
use std::char;
use std::io::{Error, ErrorKind};
use std::iter::FromIterator;
use std::ops::Index;
use rand::thread_rng;
use rand::prelude::SliceRandom;
use super::piece::{Piece, Color, Color::*};

#[derive(Copy, Clone, Hash, Eq)]
pub struct Move{
	pub from: Position,
	pub to: Position,
	heuristic_value: Score,
	pub promote: Option<Piece>
}

#[derive(PartialEq, Debug, Clone)]
pub struct Moves(Vec<Move>);

pub type Score = i32;
pub const INFINITY: Score = 2147483647;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position{
	pub x: usize,
	pub y: usize
}

impl Move{
	//Lager et nytt move. NB! Denne bryr seg kun om koordinater, der Origo er oppe til venstre.
	//Dermed er e1->e2 det samme som (4, 7, 4, 6).
	pub fn new(filefrom: usize, rankfrom: usize, fileto: usize, rankto: usize, promote: Option<Piece>, heuristic_value: Score) -> Self{
		Move{from: Position{x: filefrom, y: rankfrom}, to: Position{x: fileto, y: rankto}, promote, heuristic_value}
	}

	//Parser en streng på formen "e2e4". "e4", "e2-e4", "Pe4" er ikke gyldig og gir None.
	//For å promotere, legg til en karakter om hvilken brikke du vil promotere til på slutten, f. eks "b7a8Q".
	pub fn from_str(s: &str) -> Result<Move, Error>{
		let mut l = s.chars();
		let filefrom = l.next().ok_or_else(|| format!("Couldn't read the first character in {}, expected a-h ", s)).unwrap() as i32 - 97;
		let rankfrom = 56 - l.next().ok_or_else(|| format!("Couldn't read the second character in {}, expected 1-8", s)).unwrap() as i32;
		let fileto   = l.next().ok_or_else(|| format!("Couldn't read the third character in {}, expected a-h ", s)).unwrap() as i32 - 97;
		let rankto   = 56 - l.next().ok_or_else(|| format!("Couldn't read the fouth character in {}, expected 1-8", s)).unwrap() as i32;
		if [filefrom, rankfrom, fileto, rankto].iter().any(|i| i < &0 || i >= &8) { return Err(Error::new(ErrorKind::Other, "Characters in a move have to be a-h or 1-8")); }
		match l.next(){
			None => Ok(Move::new(filefrom as usize, rankfrom as usize, fileto as usize, rankto as usize, None, 0)),
			Some(c) => {
				if "RNBQrnbq".to_string().chars().any(|p| p == c) { 
					Ok(Move::new(filefrom as usize, rankfrom as usize, fileto as usize, rankto as usize, Piece::new(c), 0))
			 	}else{ Err(Error::new(ErrorKind::Other, "Bruh")) }
			}
		}
	}

	pub fn to_string_short(&self) -> String{
		let mut ret = String::new();
		ret.push(char::from_u32(self.from.x as u32 + 97).unwrap());
		ret.push(char::from_digit((8 - self.from.y) as u32, 10).unwrap());
		ret.push(char::from_u32(self.to.x as u32 + 97).unwrap());
		ret.push(char::from_digit((8 - self.to.y) as u32, 10).unwrap());
		ret
	}

	pub fn heuristic_value(&self) -> Score{
		self.heuristic_value
	}

	pub fn set_heuristic_value(&mut self, s: Score){
		self.heuristic_value = s;
	}
}

impl Moves{
	pub fn new() -> Self{
		Moves(Vec::new())
	}

	pub fn push(&mut self, m: Move){
		self.0.push(m);
	}

	pub fn pop(&mut self) -> Option<Move>{
		self.0.pop()
	}

	pub fn append(&mut self, other: &mut Moves){
		self.0.append(&mut other.0);
	}

	pub fn sort_by_heuristic(&mut self, c: Color){
		if c == White { self.0.sort_by(|m1, m2| m2.heuristic_value().cmp(&m1.heuristic_value())); }
		else { self.0.sort_by(|m1, m2| m1.heuristic_value().cmp(&m2.heuristic_value())); }
	}

	pub fn contains(&self, m: &Move) -> bool{
		self.0.contains(m)
	}

	pub fn len(&self) -> usize{
		self.0.len()
	}

	pub fn shuffle(&mut self){
		self.0.shuffle(&mut thread_rng());
	}

	pub fn choice(&self) -> Move{
		self.0.choose(&mut rand::thread_rng()).expect("This list of Moves wasn't supposed to be empty, but it is").clone()
	}
}

impl IntoIterator for Moves{
	type Item = Move;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter{
		self.0.into_iter()
	}
}

impl FromIterator<Move> for Moves {
    fn from_iter<I: IntoIterator<Item=Move>>(iter: I) -> Self {
        let mut c = Moves::new();
        for i in iter {
            c.push(i);
        }
        c
    }
}

impl Index<usize> for Moves {
    type Output = Move;

    fn index(&self, i: usize) -> &Self::Output {
		&self.0[i]
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

#[cfg(test)]
mod move_tests{
	use super::*;

	#[test]
	fn can_sort_by_heurestic(){
		let mut moves = Moves(vec![Move::new(0, 0, 0, 0, None, 50), Move::new(0, 0, 0, 0, None, 25), Move::new(0, 0, 0, 0, None, -10), Move::new(0, 0, 0, 0, None, 100)]);
		let expected_white = Moves(vec![Move::new(0, 0, 0, 0, None, 100), Move::new(0, 0, 0, 0, None, 50), Move::new(0, 0, 0, 0, None, 25), Move::new(0, 0, 0, 0, None, -10)]);
		moves.sort_by_heuristic(White);

		assert_eq!(moves, expected_white);

		let expected_black = Moves(vec![Move::new(0, 0, 0, 0, None, -10), Move::new(0, 0, 0, 0, None, 25), Move::new(0, 0, 0, 0, None, 50), Move::new(0, 0, 0, 0, None, 100)]);
		moves.sort_by_heuristic(Black);

		assert_eq!(moves, expected_black);
	}
}