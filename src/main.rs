mod backend;
mod ai;

use crate::backend::board;
use crate::backend::movement;
use crate::ai::minimax;
use crate::ai::interface::AI;

use std::io;

fn main() {
    let mut board = board::Board::new();
    let mut mm = minimax::MiniMax::new();
    println!("{}", board.to_string());

    
    loop{
        mm.search(board.clone());
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