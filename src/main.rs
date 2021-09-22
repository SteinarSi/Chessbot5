#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(bare_trait_objects)]

mod backend;

use backend::bot_testing::test_bot;
use backend::gameboard::Gameboard;
use crate::backend::network::Connection::*;
use std::io;
use std::str;
use std::net::*;
use futures::executor::block_on;

fn main(){
    //test_bot();
    //play();
}

fn play(){
    println!("Welcome!
Please choose how to play:
1) Play against the bot as white
2) Play against the bot as black
3) Observe the bot play against itself
4) Multiplayer");
    let mut input = String::new();
    let mut bot_next;
    let mut eve = false;
    let mut game = Gameboard::new();

    //MP Vars
    let mut is_multiplayer = false;
    let mut my_turn = true;

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
            "4"=> {
                is_multiplayer = true;
                println!("
                    1) Host a game (Play as white)
                    2) Connect to a game (Play as black)
                    ");
                io::stdin().read_line(&mut input).unwrap();
                match &input.trim()[..]{
                    "1" => {
                        my_turn = true;
                    }
                    "2" => {
                        my_turn = false;     
                    }  
                    _=>{
                        println!("Could not recognize input. Please try again.");
                    }
                }
            }
            _ => {
                println!("Could not recognize input. Please try again.");
            }
        }
    }
    input.clear();
    if is_multiplayer
    {
        println!("
                Enter IP and Port
                ");
        io::stdin().read_line(&mut input).unwrap();    

        let ip = &input;
        let mut conn = Connection::new(ip.to_string(), my_turn);

        while ! game.is_game_over(){
            println!("{}", game.to_string());
            if !my_turn{//Their turn
                println!("Waiting for other player...");
                //Wait until u recv a move
                let mut mov = block_on(conn.recv_move());
                game.move_piece(str::from_utf8(&mov).unwrap());
            }
            else{//My turn
                io::stdin().read_line(&mut input).unwrap();
                let attempt = game.move_piece(&input.trim());
                if attempt.is_ok(){
                    conn.send_move(&input);//Send ur move to them
                    my_turn = false;
                }
                else{
                    println!("{:?}", attempt);
                }
                input.clear();
            }
        }
    }

    else{
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
}
