mod backend;
mod ai;

use crate::backend::{board, movement};
use crate::ai::{interface::AI, minimax, memomax, alphabeta};

use std::io;

fn main2() {
    let mut board = board::Board::new();
    
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

fn main(){
    //memomax_vs_alphabeta();
    //compare_moves();
    /*
    let mut board = board::Board::new();
    let mut bot = memomax::MemoMax::new();

    loop{
        println!("{}", board.to_string());
        let m = bot.search(board.clone());
        println!("{}: {}", m.to_string(), m.actual_value());
        board.move_piece(&m);
    }*/
}

fn minimax_vs_memomax(){
    let mut board = board::Board::new();
    let mut mini = minimax::MiniMax::new();
    let mut memo = memomax::MemoMax::new();

    loop{
        println!("{}", board.to_string());
        let m = mini.search(board.clone());
        println!("{}: {}", m.to_string(), m.actual_value());
        board.move_piece(&m);
        println!("{}", board.to_string());
        let m = memo.search(board.clone());
        println!("{}: {}", m.to_string(), m.actual_value());
        board.move_piece(&m);
    }
}

fn memomax_vs_alphabeta(){
    let mut board = board::Board::new();
    let mut alphabeta = alphabeta::AlphaBeta::new();
    let mut memo = memomax::MemoMax::new();

    loop{
        println!("{}", board.to_string());
        let m = alphabeta.search(board.clone());
        println!("{}: {}", m.to_string(), m.actual_value());
        board.move_piece(&m);
        println!("{}", board.to_string());
        let m = memo.search(board.clone());
        println!("{}: {}", m.to_string(), m.actual_value());
        board.move_piece(&m);
    }
}

fn compare_moves(){
    println!("\nMiniMax: ");
    let mut board = board::Board::new();
    let mut mini = minimax::MiniMax::new();
    for _ in 1..=10{
        let m = mini.search(board.clone());
        print!("{}: {}, ", m.to_string(), m.actual_value());
        board.move_piece(&m);
    }

    println!("\nMemoMax: ");
    let mut board = board::Board::new();
    let mut memo = minimax::MiniMax::new();
    for _ in 1..=10{
        let m = memo.search(board.clone());
        print!("{}: {}, ", m.to_string(), m.actual_value());
        board.move_piece(&m);
    }

    println!("\nAlphaBeta: ");
    let mut board = board::Board::new();
    let mut alpha = minimax::MiniMax::new();
    for _ in 1..=10{
        let m = alpha.search(board.clone());
        print!("{}: {}, ", m.to_string(), m.actual_value());
        board.move_piece(&m);
    }
}