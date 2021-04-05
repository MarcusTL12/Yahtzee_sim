pub mod yahtzee;

use yahtzee::DiceThrow;

fn main() {
    let throw = DiceThrow::roll(6);

    println!("{}", throw);
}
