use crate::backend::piece::{Piece, Color, Color::*, PieceType::*};
use crate::backend::movement::Position;
use crate::backend::board::Board;

use rand::prelude::*;
use rand_pcg::Pcg64;

pub struct Zobrist{
	hash: i64,
	arr: [[[i64; 13]; 8]; 8],
	color_to_move: i64
}

impl Zobrist{
	pub fn new(grid: &[[Option<Piece>; 8]; 8]) -> Self{
		let mut rng = Pcg64::seed_from_u64(7); //Lucky number
		let mut arr = [[[0; 13]; 8]; 8];
		for y in 0..8{
			for x in 0..8{
				for p in 0..12{
					arr[y][x][p] = rng.gen();
				}
			}
		}
		let mut hash: i64 = rng.gen();
		let color_to_move = rng.gen();

		for y in 0..8{
			for x in 0..8{
				if let Some(p) = grid[y][x]{
					hash ^= arr[y][x][p.type_index()];
				}
			}
		}

		Zobrist{hash, arr, color_to_move}
	}

	pub fn update_xy(&mut self, p: &Piece, x: usize, y: usize){
		self.hash ^= self.arr[y][x][p.type_index()];
	}

	pub fn update_pos(&mut self, p: &Piece, pos: &Position){
		self.update_xy(p, pos.x, pos.y);
	}

	pub fn hash(&self) -> i64{
		self.hash
	}

	pub fn swap_sides(&mut self){
		self.hash ^= self.color_to_move;
	}

	
}
