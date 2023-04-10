pub mod interface;
mod structures;

//En mappe for alle bottene. Hver av dem er en oppgradering til den forrige.
//Sortert etter kompetanse:
pub mod minimax; pub mod negamax;
pub mod memomax; pub mod negamemomax;
pub mod alphabeta; pub mod negaalpha;
pub mod memoalpha; pub mod negamemoalpha;
pub mod alphakiller;
pub mod quiescence;
pub mod pvs;
pub mod iddfs;
pub mod omikron;
pub mod splitter;


#[cfg(test)]
mod negamax_tests {
    use crate::backend::board_representation::board::Board;
    use super::{interface::AI, minimax::MiniMax, negamax::NegaMax, alphabeta::AlphaBeta, negaalpha::NegaAlpha, memomax::MemoMax, negamemomax::NegaMemoMax, memoalpha::MemoAlpha, negamemoalpha::NegaMemoAlpha};

    #[test]
    fn minimax_negamax_same_move(){
        bots_return_the_same_move(&mut MiniMax::new(), &mut NegaMax::new(), 3);
    }

    #[test]
    fn alphabeta_negaalpha_same_move(){
        bots_return_the_same_move(&mut AlphaBeta::new(), &mut NegaAlpha::new(), 5);
    }

    #[test]
    fn memomax_negamemomax_same_move(){
        bots_return_the_same_move(&mut MemoMax::new(), &mut NegaMemoMax::new(), 4);
    }

    #[test]
    fn memoalpha_negamemoalpha_same_move(){
        bots_return_the_same_move(&mut MemoAlpha::new(), &mut NegaMemoAlpha::new(), 5);
    }

    fn bots_return_the_same_move(normal: &mut AI, nega: &mut AI, depth: usize){
        let mut bård = Board::new();

        for d in 1..=depth{
            normal.set_depth(d);
            nega.set_depth(d);
            let m1 = normal.search(bård.clone());
            let m2 = nega.search(bård.clone());
            assert_eq!(m1, m2, "Testing {} and {} at depth {}", normal.get_name(), nega.get_name(), d);
        }

        bård.move_str("e2e4").unwrap();
        bård.move_str("e7e5").unwrap();
        bård.move_str("e1e2").unwrap();
        
        for d in 1..=depth{
            normal.set_depth(d);
            nega.set_depth(d);
            let m1 = normal.search(bård.clone());
            let m2 = nega.search(bård.clone());
            assert_eq!(m1, m2, "Testing {} and {} at depth {}", normal.get_name(), nega.get_name(), d);
        }
    }
}