mod backend;

use crate::backend::board;
use crate::backend::movement;

use std::io;

fn main() {
    let mut board = board::Board::new();
    println!("{}", board.to_string());
    loop{
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if board.move_str(&input).is_some(){
            println!("{}", board.to_string());
        }
        else{
            println!("Nope, got an error parsing your move.");
        }
    }
}