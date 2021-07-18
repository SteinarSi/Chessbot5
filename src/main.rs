#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(bare_trait_objects)]

mod backend;
mod ai;

use crate::backend::{board::*, movement::*};
use crate::ai::{interface::AI, minimax::MiniMax, memomax::MemoMax, alphabeta::AlphaBeta, memoalpha::MemoAlpha, alphakiller::AlphaKiller, quiescence::Quiescence};
use std::io;

fn main(){
    //vs(&mut MemoMax::new(), 5, &mut Quiescence::new(), 8);
    simulate(&mut Quiescence::new(), 8);
    //play_against(&mut Quiescence::new(), 8, White);
    //compare_moves();
    //vs(&mut Quiescence::new(), 8, &mut AlphaKiller::new(), 8);
    //compare(&mut [("AlphaBeta", &mut AlphaBeta::new()), ("AlphaKiller", &mut AlphaKiller::new()), ("Quiescence", &mut Quiescence::new())], 6, 7);
}

fn play_against(bot: &mut AI, depth: usize, c: Color){
    bot.set_depth(depth);
    let mut board = Board::new();
    println!("{}", board.to_string());

    if c == Black{
        println!("Thinking...");
        let m = bot.search(board.clone());
        board.move_piece(&m);
        println!("{}", board.to_string());
    }
    
    loop{
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if board.move_str(&input).is_some(){
            println!("{}", board.to_string());
            if board.is_checkmate(){
                println!("You win!");
                break;
            }
            println!("Thinking...");
            let m = bot.search(board.clone());
            board.move_piece(&m);
            println!("{}", board.to_string());
            if board.is_checkmate() {  
                println!("Bot wins!");
                break;
            }
        }
        else{
            println!("Nope, got an error parsing your move.");
        }
    }
}

fn simulate(bot: &mut AI, depth: usize){
    let mut board = Board::new();
    bot.set_depth(depth);
    println!("{}", board.to_string());

    loop{
        let m = bot.search(board.clone());
        println!("{}", m.to_string());
        board.move_piece(&m);
        println!("{}", board.to_string());
        if board.is_checkmate(){
            println!("{} wins!", board.color_to_move().opposite());
            break;
        }
    }
}

fn vs(bot1: &mut AI, depth1: usize, bot2: &mut AI, depth2: usize){
    let mut board = Board::new();
    bot1.set_depth(depth1);
    bot2.set_depth(depth2);
    println!("{}", board.to_string());
    loop{    
        let m = bot1.search(board.clone());
        board.move_piece(&m);
        println!("{}", board.to_string());
        if board.is_checkmate(){
            println!("{} wins!", board.color_to_move().opposite());
            break;
        }

        let m = bot2.search(board.clone());
        board.move_piece(&m);
        println!("{}", board.to_string());
        if board.is_checkmate(){
            println!("{} wins!", board.color_to_move().opposite());
            break;
        }
    }
}


fn compare(l: &mut [(&str, &mut AI)], n: i8, depth: usize){
    for (name, bot) in l{
        bot.set_depth(depth);
        println!("{}:", name);
        let mut b = Board::new();
        for _ in 0..n{
            let m = bot.search(b.clone());
            print!("{}", m.to_string());
            b.move_piece(&m);
        }
        println!("\n");
    }

}

fn solve_position(s: &str, c: Color) -> Moves{
    let b = Board::custom(s, c);
    let mut killer = Quiescence::new();
    killer.set_depth(10);
    killer.principal_variation(b)

}