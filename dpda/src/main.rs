mod dpda;
mod utils;

use crate::dpda::*;
use crate::utils::*;

fn main() -> Result<(), LoadError> {
    let args: Vec<String> = std::env::args().collect();
    if (args.len() != 3 && args.len() != 4) || (args.len() == 4 && args[3] != "verbose") {
        println!("Usage: cargo run -dpda <filename> <input> [verbose]");
        return Ok(());
    }
    let filename = &args[1];
    let input = &args[2];
    let verbose = args.len() == 4 && args[3] == "verbose";
    let dpda = DPDA::load(filename)?;
    if dpda.is_accepted(input, verbose) {
        println!("\"{input}\" is \x1b[{}mAccepted\x1b[m", 32);
    } else {
        println!("\"{input}\" is \x1b[{}mRejected\x1b[m", 31);
    }

    Ok(())
}
