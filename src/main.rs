#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(bare_trait_objects)]

mod backend;

use backend::bot_testing::test_bot;

fn main(){
    test_bot();
}

