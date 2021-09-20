#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(bare_trait_objects)]

mod backend;

use backend::bot_testing::test_bot;
use backend::gameboard::Gameboard;
use std::io;

fn main(){
    //test_bot();
    play();
}

fn play(){
    println!("Welcome!
Please choose how to play:
1) Play against the bot as white
2) Play against the bot as black
3) Observe the bot play against itself");
    let mut input = String::new();
    let mut game = Gameboard::new();
    let mut bot_next;
    let mut eve = false;
    loop{
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        match &input.trim()[..]{
            "1" => {
                bot_next = false;
                break;
            }
            "2" => {
                bot_next = true;
                break;
            }
            "3" => {
                eve = true;
                bot_next = true;
                break;
            }
            _   => {
                println!("Could not recognize input. Please try again.");
            }
        }
    }
    input.clear();
    while ! game.is_game_over(){
        println!("{}", game.to_string());
        if bot_next{
            println!("Thinking....");
            game.start_bot();
            bot_next = eve;
        }
        else{
            io::stdin().read_line(&mut input).unwrap();
            let attempt = game.move_piece(&input.trim());
            if attempt.is_ok(){
                bot_next = true;
            }
            else{
                println!("{:?}", attempt);
            }
            input.clear();
        }
    }
    println!("{}", game.to_string());
    if game.is_checkmate(){
        println!("{} wins!", game.winner().unwrap());
    }else {
        println!("Draw!");
    }
}
