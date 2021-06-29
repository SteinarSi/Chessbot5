mod board;
mod piece;
mod movement;

use board::Board;

fn main() {
    let mut board = Board::new();
    println!("{}", board.to_string());
    let e2e4 = board::Move::new(4, 6, 4, 4);
    let e1e2 = board::Move::new(4, 7, 4, 6);
    board.move_piece(e2e4);
    board.move_piece(e1e2);
    println!("{}", board.to_string());

    board.go_back();
    board.go_back();
    println!("{}", board.to_string());
}
