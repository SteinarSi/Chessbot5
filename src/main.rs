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
    simulate(&mut Quiescence::new(), 8);
    //play_against(&mut Quiescence::new(), 8, Black);
    //compare_moves();
    //vs(&mut Quiescence::new(), &mut AlphaKiller::new(), 8);
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
            println!("Thinking...");
            let m = bot.search(board.clone());
            board.move_piece(&m);
            println!("{}", board.to_string());
        }
        else{
            println!("Nope, got an error parsing your move.");
        }
    }
}

fn simulate(bot: &mut AI, depth: usize){
    let mut board = Board::new();
    bot.set_depth(depth);

    loop{
        println!("{}", board.to_string());
        let m = bot.search(board.clone());
        println!("{}", m.to_string());
        board.move_piece(&m);
    }
}

fn vs(bot1: &mut AI, bot2: &mut AI, depth: usize){
    let mut board = Board::new();
    bot1.set_depth(depth);
    bot2.set_depth(depth);
    loop{    
        println!("{}", board.to_string());
        let m = bot1.search(board.clone());
        board.move_piece(&m);

        println!("{}", board.to_string());
        let m = bot2.search(board.clone());
        board.move_piece(&m);
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