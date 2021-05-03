use std::{
    fmt::Display,
    io::{stdin, stdout, Write},
    iter::Sum,
    vec,
};

use num_traits::Num;

use crate::{
    yahtzee_free_strats::{get_cell_strat, get_rethrow_strat, get_score},
    yahtzee_strats::{get_index_name, new_throw},
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

fn tostr<T: Num + Display + PartialEq + Clone>(
    points: &Vec<Option<T>>,
    ind: &mut usize,
) -> String {
    let ans = match points[*ind].clone() {
        None => "".to_owned(),
        Some(x) if x == T::zero() => "-".to_owned(),
        Some(x) => format!("{}", x),
    };
    *ind += 1;
    ans
}

pub fn display_points<
    T: Num + Display + Sum + PartialOrd + PartialEq + Clone,
    const N: u64,
>(
    points: &Vec<Option<T>>,
    prec_bonus: Option<T>,
    prec_sum: Option<T>,
) {
    let ind = &mut 0;
    println!("ones              = {}", tostr(points, ind));
    println!("twos              = {}", tostr(points, ind));
    println!("threes            = {}", tostr(points, ind));
    println!("fours             = {}", tostr(points, ind));
    println!("fives             = {}", tostr(points, ind));
    println!("sixes             = {}", tostr(points, ind));
    println!("------------------------------------");
    let above: T = points.iter().take(6).filter_map(|x| x.clone()).sum();
    let bonus_objective = match N {
        5 => 63,
        6 => 84,
        _ => unreachable!(),
    };
    let bonus_objective: T = (0..bonus_objective).map(|_| T::one()).sum();

    let bonus = if above >= bonus_objective {
        match N {
            5 => 50,
            6 => 100,
            _ => unreachable!(),
        }
    } else {
        0
    };

    let bonus: T = if let Some(b) = prec_bonus {
        b
    } else {
        (0..bonus).map(|_| T::one()).sum()
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
        if let Some(s) = prec_sum {
            s
        } else {
            bonus + points.iter().filter_map(|x| x.clone()).sum()
        }
    );
}

fn get_total_score<const N: u64>(points: &[Option<u64>]) -> u64 {
    let points_above: u64 =
        points.iter().take(6).filter_map(|x| x.as_ref()).sum();

    let bonus_objective = match N {
        5 => 63,
        6 => 84,
        _ => unreachable!(),
    };

    let bonus = if points_above >= bonus_objective {
        match N {
            5 => 50,
            6 => 100,
            _ => unreachable!(),
        }
    } else {
        0
    };

    let total = bonus + points.iter().filter_map(|x| x.as_ref()).sum::<u64>();

    total
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

    let mut last_dice = DiceThrow::throw(N as usize);
    let mut throws_left = 2;

    println!("Starting throw:\n{}", last_dice);

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
            ["display", "points"] => {
                display_points::<_, N>(&points, None, None)
            }
            ["set", "points", cell, pts] => {
                let index = super::get_yahtzee_index::<N>(cell);
                let pts = pts.parse().unwrap();
                points[index] = Some(pts);
                display_points::<_, N>(&points, None, None);
            }
            ["clear", "points", cell] => {
                let index = super::get_yahtzee_index::<N>(cell);

                points[index] = None;
            }
            ["throw", "dice", n] => {
                let n = n.parse().unwrap();

                let throw = DiceThrow::throw(n);

                println!("{}", throw);

                last_dice = throw;
            }
            ["auto"] => {
                let free_cells: Vec<_> =
                    points.iter().map(|x| x.is_none()).collect();

                let points_above =
                    points.iter().take(6).filter_map(|x| x.as_ref()).sum();

                if throws_left == 0 {
                    let ind = get_cell_strat::<N>(
                        &free_cells,
                        &last_dice,
                        points_above,
                    );

                    let score = last_dice.cell_score::<N>(ind);

                    println!(
                        "Putting {} points in {}.",
                        score,
                        get_index_name::<N>(ind)
                    );

                    points[ind] = Some(score);
                    display_points::<_, N>(&points, None, None);

                    last_dice = DiceThrow::throw(N as usize);
                    throws_left = 2;

                    println!("New throw:\n{}", last_dice);
                } else {
                    let rethrow = get_rethrow_strat::<N>(
                        &free_cells,
                        &last_dice,
                        throws_left,
                        points_above,
                    );

                    println!("Rethrowing:\n{}", rethrow);

                    let th = DiceThrow::throw(rethrow.amt_dice() as usize);

                    last_dice = new_throw(&last_dice, &rethrow, &th);

                    println!("To give:\n{}", last_dice);
                    throws_left -= 1;
                }
            }
            ["advise", dice_left, dice] => {
                let throws_left: usize = dice_left.parse().unwrap();
                if dice.len() != N as usize {
                    continue;
                }
                let mut throw = DiceThrow::from([0; 6]);
                for c in dice.chars() {
                    let i = (c as u8 - b'0') as u64;
                    throw[i] += 1;
                }

                println!("You entered:\n{}\n", throw);

                let free_cells: Vec<_> =
                    points.iter().map(|x| x.is_none()).collect();

                let points_above =
                    points.iter().take(6).filter_map(|x| x.as_ref()).sum();

                match throws_left {
                    0 => {
                        let ind = get_cell_strat::<N>(
                            &free_cells,
                            &throw,
                            points_above,
                        );

                        let score = throw.cell_score::<N>(ind);

                        println!(
                            "Put {} points in {}.",
                            score,
                            get_index_name::<N>(ind)
                        );
                    }
                    1 | 2 => {
                        let rethrow = get_rethrow_strat::<N>(
                            &free_cells,
                            &throw,
                            throws_left,
                            points_above,
                        );

                        println!("Rethrow:\n{}", rethrow);
                    }
                    _ => unreachable!(),
                }
            }
            ["expected-remaining"] => {
                let free_cells: Vec<_> =
                    points.iter().map(|x| x.is_none()).collect();
                let points_above =
                    points.iter().take(6).filter_map(|x| x.as_ref()).sum();
                let rem_score = get_score::<N>(
                    &free_cells,
                    &last_dice,
                    points_above,
                    throws_left,
                );

                println!("expected remaining score is {}", rem_score);
            }
            ["expected-total"] => {
                let free_cells: Vec<_> =
                    points.iter().map(|x| x.is_none()).collect();
                let points_above =
                    points.iter().take(6).filter_map(|x| x.as_ref()).sum();
                let rem_score = get_score::<N>(
                    &free_cells,
                    &last_dice,
                    points_above,
                    throws_left,
                );

                let tot_score =
                    get_total_score::<N>(&points) as f32 + rem_score;

                println!("expected total score is {}", tot_score);
            }
            ["reset"] => {
                points = vec![
                    None;
                    match N {
                        5 => 15,
                        6 => 20,
                        _ => unreachable!(),
                    }
                ];
                last_dice = DiceThrow::throw(N as usize);
                throws_left = 2;

                println!("Starting throw:\n{}", last_dice);
            }
            _ => println!("Invalid command! {:?}", command),
        }
    }
}
