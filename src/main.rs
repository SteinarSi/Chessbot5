mod board;
mod piece;

use board::Board;

fn main() {
    let mut board = Board::new();
    println!("{}", board.to_string());
    let e2e4 = board::Move::new(4, 6, 4, 4);
    board.move_piece(e2e4);
    println!("{}", board.to_string());
}
