use crate::backend::piece::{Piece, Color, Color::*, PieceType::*};
use crate::backend::movement::Position;
use crate::backend::board::Board;

use rand::prelude::*;
use rand_pcg::Pcg64;

#[derive(Clone)]
pub struct Zobrist{
	hash: i64,
	arr: [[[i64; 13]; 8]; 8],
	color_to_move: i64,
	castle: (i64, i64, i64, i64),
	en_passant: [i64; 8],
}

impl Zobrist{
	pub fn new(grid: &[[Option<Piece>; 8]; 8], color: Color) -> Self{
		let mut rng = Pcg64::seed_from_u64(7); //Lucky number

		let mut arr = [[[0; 13]; 8]; 8];
		let mut hash: i64 = rng.gen();
		let color_to_move = rng.gen();
		let castle = (rng.gen(), rng.gen(), rng.gen(), rng.gen());
		let en_passant = &[rng.gen(), rng.gen(), rng.gen(), rng.gen(), rng.gen(), rng.gen(), rng.gen(), rng.gen()];

		for y in 0..8{
			for x in 0..8{
				for p in 0..12{
					arr[y][x][p] = rng.gen();
				}
			}
		}

		for y in 0..8{
			for x in 0..8{
				if let Some(p) = grid[y][x]{
					hash ^= arr[y][x][p.type_index()];
				}
			}
		}
		if color == Black { hash ^= color_to_move; }

		Zobrist{hash, arr, color_to_move, castle, en_passant: *en_passant}
	}

	pub fn update_xy(&mut self, x: usize, y: usize, p: &Piece){
		self.hash ^= self.arr[y][x][p.type_index()];
	}

	pub fn update_pos(&mut self, pos: &Position, p: &Piece){
		self.update_xy(pos.x, pos.y, p);
	}

	pub fn hash(&self) -> i64{
		self.hash
	}

	pub fn set_hash(&mut self, new_hash: i64){
		self.hash = new_hash;
	}

	pub fn swap_sides(&mut self){
		self.hash ^= self.color_to_move;
	}

	pub fn update_castle(&mut self, castle: (bool, bool, bool, bool)){
		if castle.0 {
			self.hash ^= self.castle.0;
		}
		if castle.1 {
			self.hash ^= self.castle.1;
		}
		if castle.2 {
			self.hash ^= self.castle.2;
		}
		if castle.3 {
			self.hash ^= self.castle.3;
		}	
	}

	pub fn update_en_passant(&mut self, c: usize){
		self.hash ^= self.en_passant[c];
	}
}

impl PartialEq for Zobrist{
	fn eq(&self, other: &Zobrist) -> bool{
		self.hash == other.hash
	}
}
