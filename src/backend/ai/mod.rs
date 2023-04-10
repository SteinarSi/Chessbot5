pub mod interface;
mod structures;

//En mappe for alle bottene. Hver av dem er en oppgradering til den forrige.
//Sortert etter kompetanse:
pub mod minimax; pub mod negamax;
pub mod memomax; pub mod negamemomax;
pub mod alphabeta; pub mod negaalpha;
pub mod memoalpha; pub mod negamemoalpha;
pub mod alphakiller; pub mod negaalphakiller;
pub mod quiescence; pub mod negaquiscence;
pub mod pvs; pub mod negapvs;
pub mod iddfs; pub mod negaiddfs;
pub mod omikron; pub mod negadb;
pub mod splitter;




#[cfg(test)]
mod negamax_tests {
    use crate::backend::board_representation::board::Board;
    use super::{interface::AI, 
                minimax::MiniMax, 
                negamax::NegaMax, 
                alphabeta::AlphaBeta, 
                negaalpha::NegaAlpha, 
                memomax::MemoMax, 
                negamemomax::NegaMemoMax, 
                memoalpha::MemoAlpha, 
                negamemoalpha::NegaMemoAlpha, 
                alphakiller::AlphaKiller, 
                negaalphakiller::NegaAlphaKiller, 
                quiescence::Quiescence, 
                negaquiscence::NegaQuiescence, 
                pvs::PVS, 
                negapvs::NegaPVS, 
                iddfs::IDDFS, 
                negaiddfs::NegaIDDFS
            };

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

    #[test]
    fn killeralpha_negaalphakiller_same_move(){
        bots_return_the_same_move(&mut AlphaKiller::new(), &mut NegaAlphaKiller::new(), 6);
    }

    #[test]
    fn quiscence_negaquiescence_same_move(){
        bots_return_the_same_move(&mut Quiescence::new(), &mut NegaQuiescence::new(), 6);
    }

    #[test]
    fn pvs_negapvs_same_move(){
        bots_return_the_same_move(&mut PVS::new(), &mut NegaPVS::new(), 6);
    }


    // This test is slightly unstable, because of time being sort of non-deterministic at times. If it fails once or twice, but runs most of the time, it's probably fine.
    #[test]
    fn iddfs_negaiddfs_same_move(){
        let mut iddfs = IDDFS::new();
        let mut negaiddfs = NegaIDDFS::new();
        iddfs.set_time(1);
        negaiddfs.set_time(1);
        bots_return_the_same_move(&mut iddfs, &mut negaiddfs, 6);
    }

    fn bots_return_the_same_move(normal: &mut AI, nega: &mut AI, depth: usize){
        let mut bård = Board::new();

        for d in 2..=depth{
            normal.set_depth(d);
            nega.set_depth(d);
            let m1 = normal.search(bård.clone());
            let m2 = nega.search(bård.clone());
            assert_eq!(m1, m2, "Testing {} and {} at depth {}", normal.get_name(), nega.get_name(), d);
        }

        bård.move_str("e2e4").unwrap();
        bård.move_str("e7e5").unwrap();
        bård.move_str("e1e2").unwrap();
        
        for d in 2..=depth{
            normal.set_depth(d);
            nega.set_depth(d);
            let m1 = normal.search(bård.clone());
            let m2 = nega.search(bård.clone());
            assert_eq!(m1, m2, "Testing {} and {} at depth {}", normal.get_name(), nega.get_name(), d);
        }
    }
}