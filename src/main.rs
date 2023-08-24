// the wheat cli

use std::env;
use howtolearn::wheat;

fn main() {
    println!("Hello, world!");

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        // println!("The first argument is {}", args[1]);

        wheat::load(args[1].to_string());
    }
}
