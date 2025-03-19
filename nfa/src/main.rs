mod nfa;
mod utils;

use crate::nfa::*;
use crate::utils::*;

fn main() -> Result<(), LoadError> {
    let args: Vec<String> = std::env::args().collect();
    if (args.len() != 3 && args.len() != 4) || (args.len() == 4 && args[3] != "verbose") {
        println!("Usage: cargo run -p nfa <filename> <input>");
        return Ok(());
    }
    let filename = &args[1];
    let input = &args[2];
    let verbose = args.len() == 4 && args[3] == "verbose";
    let nfa = NFA::load(filename)?;
    if nfa.is_accepted(input, verbose) {
        println!("\"{input}\" is \x1b[{}mAccepted\x1b[m", 32);
    } else {
        println!("\"{input}\" is \x1b[{}mRejected\x1b[m", 31);
    }

    Ok(())
}
