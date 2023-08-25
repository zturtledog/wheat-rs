// the wheat cli

use std::env;
use howtolearn::wheat;

fn main() {
    // println!("Hello, world!");

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        // println!("The first argument is {}", args[1]);

        let tofu = wheat::load(args[1].to_string());

        // println!("---- {} ----", tofu.src.to_string());
        // // wheat::dumptokens(&tofu.ast, false);
        // println!("\n-----{}-----","-".repeat(tofu.src.len()))
    }
}
