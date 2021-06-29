mod backend;

use crate::backend::board;
use crate::backend::movement;

fn main() {
    let m1 = movement::Move::from_str("e2e4").unwrap();
    let m2 = movement::Move::from_str("c7c5").unwrap();
    let mut board = board::Board::new();

    board.move_piece(m1);
    board.move_piece(m2);

    println!("{}, {}, {}", board.to_string(), m1.to_string(), m2.to_string());
}