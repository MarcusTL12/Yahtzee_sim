// #![allow(incomplete_features)]
// #![feature(const_generics)]
// #![feature(const_evaluatable_checked)]

use core::panic;
use std::env;

use yahtzee::DiceThrow;
use yahtzee_free_strats::test;
use yahtzee_guide::start;
use yahtzee_strats::{
    cache_all_tables, get_yahtzee_index, load_all_tables, make_all_tables,
};

pub mod yahtzee;
pub mod yahtzee_free_strats;
pub mod yahtzee_guide;
pub mod yahtzee_strats;
pub mod bitfield_array_file;

const HELP_MSG: &str = r#"
commands:
help: show this message
compute-strats <N>: compute and cache the strats for <N> dice
give-best-roll <N> <cell> <throws-left> <dice>: gives the best roll for a
    the given cell and dice. Write <dice> as 314156; order does not matter.
    For list of cell names run command: help-cell-names.
guide-free-game <N>: Starts an interactive session to guide through free game
    with <N> dice.
test: current test
"#;

pub const HELP_CELL_NAMES: &str = r#"
ones/enere...       => 1s - 6s
pairs/par           => 1p - 3p
of a kind / like    => 1l - 5l
straight            => ls, ss, fs
hut/hytte           => ht
house/hus           => hs
tower/tårn          => tr
chance/sjangse      => ch/sj
yahtzee/yatzy       => yz
"#;

fn comp_stats<const N: u64>() {
    cache_all_tables::<N>(make_all_tables::<N>());
}

fn give_best_roll<const N: u64>(cell: &str, throws_left: usize, dice: &str) {
    let (scores, strats) = load_all_tables::<N>();

    let mut throw = DiceThrow::from([0; 6]);
    for c in dice.chars() {
        let i = (c as u8 - b'0') as u64;
        throw[i] += 1;
    }

    println!("Your throw:\n{}\n", throw);

    let cell_ind = get_yahtzee_index::<N>(cell);

    let sub_throw = strats[throws_left][cell_ind].get(&throw).unwrap();

    println!(
        "Rethrow:\n{}\nwith expected score of: {}",
        sub_throw,
        scores[throws_left][cell_ind].get(&throw).unwrap()
    );
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if let Some(command) = args.get(1) {
        match command.as_str() {
            "help" => println!("{}", HELP_MSG),
            "compute-strats" => {
                match &args.get(2).and_then(|x| Some(x.as_str())) {
                    Some("5") => comp_stats::<5>(),
                    Some("6") => comp_stats::<6>(),
                    None => panic!("Must give number of dice (5/6)!"),
                    _ => unimplemented!("Invalid number of dice!"),
                }
            }
            "give-best-roll" => {
                match &args.get(2).and_then(|x| Some(x.as_str())) {
                    Some("5") => give_best_roll::<5>(
                        args[3].as_str(),
                        args[4].parse().unwrap(),
                        args[5].as_str(),
                    ),
                    Some("6") => give_best_roll::<6>(
                        args[3].as_str(),
                        args[4].parse().unwrap(),
                        args[5].as_str(),
                    ),
                    None => panic!("Must give number of dice (5/6)!"),
                    _ => unimplemented!("Invalid number of dice!"),
                }
            }
            "help-cell-names" => println!("{}", HELP_CELL_NAMES),
            "guide-free-game" => {
                match &args.get(2).and_then(|x| Some(x.as_str())) {
                    Some("5") => start::<5>(),
                    Some("6") => start::<6>(),
                    None => panic!("Must give number of dice (5/6)!"),
                    _ => unimplemented!("Invalid number of dice!"),
                }
            }
            "test" => test(),
            _ => unimplemented!("Invalid command: {}!", command),
        };
    } else {
        panic!("No command given");
    }
}
