use std::{
    io::{stdin, stdout, Write},
    vec,
};

use super::{
    load_all_tables,
    yahtzee_strats::{find_best_cell, get_index_name},
    DiceThrow, HELP_CELL_NAMES,
};

const HELP_MSG: &str = r#"
commands:
help: displays this message
exit/q: exit
display points: display your points
set points <cell> <points>: set a cell to a value. Get cell names by
    help cell names
clear points <cell>: clears points
advise <dice-left> <dice>: gives advice on what to do with the dice
throw dice <N>: prints a dice throw of <N> dice
"#;

fn tostr(points: &Vec<Option<u64>>, ind: &mut usize) -> String {
    let ans = match points[*ind] {
        None => "".to_owned(),
        Some(0) => "-".to_owned(),
        Some(x) => format!("{}", x),
    };
    *ind += 1;
    ans
}

fn display_points<const N: u64>(points: &Vec<Option<u64>>) {
    let ind = &mut 0;
    println!("ones              = {}", tostr(points, ind));
    println!("twos              = {}", tostr(points, ind));
    println!("threes            = {}", tostr(points, ind));
    println!("fours             = {}", tostr(points, ind));
    println!("fives             = {}", tostr(points, ind));
    println!("sixes             = {}", tostr(points, ind));
    println!("------------------------------------");
    let above = points
        .iter()
        .take(6)
        .filter_map(|x| x.as_ref())
        .sum::<u64>();
    let bonus_objective = match N {
        5 => 63,
        6 => 84,
        _ => unreachable!(),
    };

    let bonus = if above >= bonus_objective {
        match N {
            5 => 50,
            6 => 100,
            _ => unreachable!(),
        }
    } else {
        0
    };

    println!("sum               = {}", above);
    println!("bonus             = {}", bonus);
    println!("1 pair            = {}", tostr(points, ind));
    println!("2 pair            = {}", tostr(points, ind));
    if N == 6 {
        println!("3 pair            = {}", tostr(points, ind));
    }
    println!("3 of a kind       = {}", tostr(points, ind));
    println!("4 of a kind       = {}", tostr(points, ind));
    if N == 6 {
        println!("5 of a kind       = {}", tostr(points, ind));
    }
    println!("small straight    = {}", tostr(points, ind));
    println!("large straight    = {}", tostr(points, ind));
    if N == 6 {
        println!("full straight     = {}", tostr(points, ind));
        println!("hut               = {}", tostr(points, ind));
    }
    println!("house             = {}", tostr(points, ind));
    if N == 6 {
        println!("tower             = {}", tostr(points, ind));
    }
    println!("chance            = {}", tostr(points, ind));
    println!("yahtzee           = {}", tostr(points, ind));
    println!("------------------------------------");
    println!(
        "Total             = {}\n",
        bonus + points.iter().filter_map(|x| x.as_ref()).sum::<u64>()
    );
}

pub fn start<const N: u64>() {
    println!(
        "Welcome to the interactive guide of a free game with {} dice",
        N
    );

    let mut points = vec![
        None;
        match N {
            5 => 15,
            6 => 20,
            _ => unreachable!(),
        }
    ];

    let (scores, strats) = load_all_tables::<N>();

    loop {
        print!("> ");
        stdout().flush().unwrap();
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();

        let command: Vec<_> = buffer.split_whitespace().collect();

        match command.as_slice() {
            ["help"] => println!("{}", HELP_MSG),
            ["help", "cell", "names"] => println!("{}", HELP_CELL_NAMES),
            ["exit" | "q"] => break,
            ["display", "points"] => display_points::<N>(&points),
            ["set", "points", cell, pts] => {
                let index = super::get_yahtzee_index::<N>(cell);
                let pts = pts.parse().unwrap();
                points[index] = Some(pts);
                display_points::<N>(&points);
            }
            ["clear", "points", cell] => {
                let index = super::get_yahtzee_index::<N>(cell);

                points[index] = None;
            }
            ["throw", "dice", n] => {
                let n = n.parse().unwrap();

                let throw = DiceThrow::throw(n);

                println!("{}", throw);
            }
            ["advise", dice_left, dice] => {
                let dice_left: usize = dice_left.parse().unwrap();
                if dice.len() != N as usize {
                    continue;
                }
                let mut throw = DiceThrow::from([0; 6]);
                for c in dice.chars() {
                    let i = (c as u8 - b'0') as u64;
                    throw[i] += 1;
                }

                let best_ind =
                    find_best_cell::<N>(&scores[dice_left], &throw, &points);

                println!("You entered:\n{}\n", throw);

                if dice_left > 0 {
                    let sub_throw =
                        strats[dice_left][best_ind].get(&throw).unwrap();
                    println!("Reroll:\n{}", sub_throw);
                    println!(
                        "Going for {} with expected score: {}",
                        get_index_name::<N>(best_ind),
                        scores[dice_left][best_ind].get(&throw).unwrap()
                    );
                } else {
                    println!(
                        "Put {} points in {}.",
                        scores[dice_left][best_ind].get(&throw).unwrap(),
                        get_index_name::<N>(best_ind),
                    );
                }
            }
            _ => println!("Invalid command! {:?}", command),
        }
    }
}
