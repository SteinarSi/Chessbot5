use crate::backend::{board, movement};
use crate::ai::{interface::AI, minimax, memomax, alphabeta};
use crate::backend::board::*;
use crate::alphabeta::AlphaBeta;
use crate::memomax::MemoMax;
use crate::ai::memoalpha;

#[cfg(test)]
mod bot_tests{
	use super::*;

	#[ignore]
	#[test]
	fn alpha_and_memo_get_same_results(){
		let mut board = Board::new();

		let mut alpha = alphabeta::AlphaBeta::new();
		let mut memo  = memomax::MemoMax::new();

		alpha.set_depth(4);
		memo.set_depth(4);

		for _ in 1..=8{
			let m1 = alpha.search(board.clone());
			let m2 = memo.search(board.clone());
			println!("{}: {}  vs  {}:{}", m1.to_string(), m1.actual_value(), m2.to_string(), m2.actual_value());
			assert_eq!(m1, m2);

			board.move_piece(&m1);
		}
	}
	#[ignore]
	#[test]
	fn simulate_memoalpha(){
	    let mut board = Board::new();
	    let mut bot = memoalpha::MemoAlpha::new();

	    for _ in 1..=10{
	        println!("{}", board.to_string());
	        let m = bot.search(board.clone());
	        println!("{}", m.to_string());
	        board.move_piece(&m);
	    }
	}

	#[ignore]
	#[test]
	fn memomax_and_memoalpha_get_same_results(){
		let mut b = Board::new();

		let mut max = memomax::MemoMax::new();
		let mut alp = memoalpha::MemoAlpha::new();

		for _ in 1..=10{
			let m1 = alp.search(b.clone());
			let m2 = max.search(b.clone());

			println!("{}  vs  {}", m1.to_string(), m2.to_string());
			assert_eq!(m1, m2);

			b.move_piece(&m1);
		}
	}
	#[ignore]
	#[test]
	fn memoalpha_and_alphabeta_get_same_results(){
		let mut b = Board::new();

		let mut alpha = alphabeta::AlphaBeta::new();
		alpha.set_depth(5);
		let mut memo = memoalpha::MemoAlpha::new();
		memo.set_depth(5);

		for _ in 1..=10{
			let m1 = alpha.search(b.clone());
			let m2 = memo.search(b.clone());

			println!("{}  vs  {}", m1.to_string(), m2.to_string());
			assert_eq!(m1, m2);

			b.move_piece(&m1);
		}	
	}

	#[ignore]
	#[test]
	fn memoalpha_principal_variation(){
		let b = Board::new();
		let mut memoalpha = memoalpha::MemoAlpha::new();

		for m in memoalpha.principal_variation(b){
			println!("{}", m.to_string());
		}
	}
}