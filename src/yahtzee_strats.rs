use std::{
    collections::HashMap,
    fs::{self, create_dir_all, write},
};

use super::yahtzee::DiceThrow;

pub struct DiceIter {
    done: bool,
    dice: DiceThrow,
}

impl DiceIter {
    pub fn new(n: u64) -> Self {
        Self {
            done: false,
            dice: DiceThrow::from([n, 0, 0, 0, 0, 0]),
        }
    }
}

impl Iterator for DiceIter {
    type Item = DiceThrow;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else if let Some(i) = (1..=5).find(|&i| self.dice[i] > 0) {
            let ans = Some(self.dice.clone());
            self.dice[i] -= 1;
            self.dice[i + 1] += 1;
            if i > 1 {
                self.dice[1] = self.dice[i];
                self.dice[i] = 0;
            }

            ans
        } else {
            let ans = Some(self.dice.clone());
            self.dice[1] = self.dice[6];
            self.dice[6] = 0;

            self.done = true;

            ans
        }
    }
}

fn new_throw(
    orig_dice: &DiceThrow,
    sub_throw: &DiceThrow,
    new: &DiceThrow,
) -> DiceThrow {
    let mut dice = orig_dice.clone();
    for i in 1..=6 {
        dice[i] -= sub_throw[i];
        dice[i] += new[i];
    }
    dice
}

pub fn re_throw_iters<'a>(
    orig_dice: &'a DiceThrow,
    sub_throw: &'a DiceThrow,
) -> impl Iterator<Item = (DiceThrow, f64)> + 'a {
    let amt_dice: u64 = (1..=6).map(|i| sub_throw[i]).sum();
    DiceIter::new(amt_dice).map(move |new| {
        (new_throw(orig_dice, &sub_throw, &new), new.probability())
    })
}

pub fn expected_score(
    orig_dice: &DiceThrow,
    sub_throw: &DiceThrow,
    scores: &HashMap<DiceThrow, f64>,
) -> f64 {
    re_throw_iters(orig_dice, sub_throw)
        .map(|(throw, prob)| scores.get(&throw).unwrap() * prob)
        .sum()
}

pub fn make_score_table<F: Fn(&DiceThrow) -> f64>(
    f: F,
    n: u64,
) -> HashMap<DiceThrow, f64> {
    DiceIter::new(n)
        .map(|throw| (throw.clone(), f(&throw)))
        .collect()
}

pub fn make_init_score_tables<const N: u64>() -> Vec<HashMap<DiceThrow, f64>> {
    let mut tables = Vec::new();

    tables.push(make_score_table(|throw| throw.ammount_of::<1>() as f64, N));
    tables.push(make_score_table(|throw| throw.ammount_of::<2>() as f64, N));
    tables.push(make_score_table(|throw| throw.ammount_of::<3>() as f64, N));
    tables.push(make_score_table(|throw| throw.ammount_of::<4>() as f64, N));
    tables.push(make_score_table(|throw| throw.ammount_of::<5>() as f64, N));
    tables.push(make_score_table(|throw| throw.ammount_of::<6>() as f64, N));

    tables.push(make_score_table(|throw| throw.pairs::<1>() as f64, N));
    tables.push(make_score_table(|throw| throw.pairs::<2>() as f64, N));
    if N == 6 {
        tables.push(make_score_table(|throw| throw.pairs::<3>() as f64, N));
    }

    tables.push(make_score_table(|throw| throw.n_of_a_kind::<3>() as f64, N));
    tables.push(make_score_table(|throw| throw.n_of_a_kind::<4>() as f64, N));
    if N == 6 {
        tables
            .push(make_score_table(|throw| throw.n_of_a_kind::<5>() as f64, N));
    }

    tables.push(make_score_table(|throw| throw.straight::<1, 5>() as f64, N));
    tables.push(make_score_table(|throw| throw.straight::<2, 6>() as f64, N));
    if N == 6 {
        tables
            .push(make_score_table(|throw| throw.straight::<1, 6>() as f64, N));
    }

    tables.push(make_score_table(|throw| throw.building::<3, 2>() as f64, N));
    if N == 6 {
        tables
            .push(make_score_table(|throw| throw.building::<3, 3>() as f64, N));
        tables
            .push(make_score_table(|throw| throw.building::<4, 2>() as f64, N));
    }

    tables.push(make_score_table(|throw| throw.chance() as f64, N));
    tables.push(make_score_table(|throw| throw.yahtzee() as f64, N));

    tables
}

pub fn make_strat_from_score_table(
    table: &HashMap<DiceThrow, f64>,
) -> HashMap<DiceThrow, DiceThrow> {
    table
        .keys()
        .map(|throw| {
            (
                throw.clone(),
                throw
                    .clone()
                    .into_sub_throw_iter()
                    .map(|sub_throw| {
                        (
                            sub_throw.clone(),
                            expected_score(throw, &sub_throw, &table),
                        )
                    })
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .unwrap()
                    .0,
            )
        })
        .collect()
}

pub fn make_next_score_table(
    prev_table: &HashMap<DiceThrow, f64>,
    prev_strat: &HashMap<DiceThrow, DiceThrow>,
) -> HashMap<DiceThrow, f64> {
    prev_table
        .keys()
        .map(|throw| {
            (
                throw.clone(),
                expected_score(
                    throw,
                    prev_strat.get(&throw).unwrap(),
                    prev_table,
                ),
            )
        })
        .collect()
}

pub fn make_all_tables<const N: u64>() -> (
    Vec<Vec<HashMap<DiceThrow, f64>>>,
    Vec<Vec<HashMap<DiceThrow, DiceThrow>>>,
) {
    let mut scores = vec![make_init_score_tables::<N>()];

    let mut strats: Vec<Vec<_>> = vec![Vec::new()];

    for _ in 0..2 {
        strats.push(
            scores
                .last()
                .unwrap()
                .iter()
                .map(make_strat_from_score_table)
                .collect(),
        );

        scores.push(
            scores
                .last()
                .unwrap()
                .iter()
                .zip(strats.last().unwrap().iter())
                .map(|(score, strat)| make_next_score_table(score, strat))
                .collect(),
        );
    }

    (scores, strats)
}

pub fn cache_all_tables<const N: u64>(
    vals: (
        Vec<Vec<HashMap<DiceThrow, f64>>>,
        Vec<Vec<HashMap<DiceThrow, DiceThrow>>>,
    ),
) {
    create_dir_all("bincode/").unwrap();
    write(
        format!("bincode/strats{}.bincode", N),
        bincode::serialize(&vals).unwrap(),
    )
    .unwrap();
}

pub fn load_all_tables<const N: u64>() -> (
    Vec<Vec<HashMap<DiceThrow, f64>>>,
    Vec<Vec<HashMap<DiceThrow, DiceThrow>>>,
) {
    bincode::deserialize(
        &fs::read(format!("bincode/strats{}.bincode", N)).unwrap(),
    )
    .unwrap()
}

pub fn get_yahtzee_index<const N: u64>(name: &str) -> usize {
    match (N, name) {
        (_, "1s") => 0,
        (_, "2s") => 1,
        (_, "3s") => 2,
        (_, "4s") => 3,
        (_, "5s") => 4,
        (_, "6s") => 5,
        (_, "1p") => 6,
        (_, "2p") => 7,
        (6, "3p") => 8,
        (5, "3l") => 8,
        (6, "3l") => 9,
        (5, "4l") => 9,
        (6, "4l") => 10,
        (6, "5l") => 11,
        (5, "ls") => 10,
        (6, "ls") => 12,
        (5, "ss") => 11,
        (6, "ss") => 13,
        (6, "fs") => 14,
        (5, "hs") => 12,
        (6, "ht") => 15,
        (6, "hs") => 16,
        (6, "tr") => 17,
        (5, "ch" | "sj") => 13,
        (6, "ch" | "sj") => 18,
        (5, "yz") => 14,
        (6, "yz") => 19,
        _ => unreachable!(),
    }
}

pub fn get_index_name<const N: u64>(ind: usize) -> &'static str {
    match (N, ind) {
        (_, 0) => "ones",
        (_, 1) => "twos",
        (_, 2) => "threes",
        (_, 3) => "fours",
        (_, 4) => "fives",
        (_, 5) => "sixes",
        (_, 6) => "1 pair",
        (_, 7) => "2 pairs",
        (6, 8) => "3 pairs",
        (5, 8) | (6, 9) => "3 of a kind",
        (5, 9) | (6, 10) => "4 of a kind",
        (6, 11) => "5 of a kind",
        (5, 10) | (6, 12) => "small straight",
        (5, 11) | (6, 13) => "large straight",
        (6, 14) => "full straight",
        (6, 15) => "hut",
        (5, 12) | (6, 16) => "house",
        (6, 17) => "tower",
        (5, 13) | (6, 18) => "chance",
        (5, 14) | (6, 19) => "yahtzee",
        _ => unreachable!(),
    }
}

pub fn effective_score<const N: u64>(
    scores: &Vec<HashMap<DiceThrow, f64>>,
    throw: &DiceThrow,
    points_above: u64,
    cell_ind: usize,
) -> f64 {
    let bonus_bias = match N {
        5 => 1.0,
        6 => 1.5,
        _ => unreachable!(),
    };

    let bonus_offset = match N {
        5 => 1.0,
        6 => 1.5,
        _ => unreachable!(),
    };

    let bonus_objective = (cell_ind + 1) as f64
        * match N {
            5 => 3.0,
            6 => 4.0,
            _ => unreachable!(),
        };

    let score = scores[cell_ind].get(throw).unwrap();

    score
        * if cell_ind <= 5 {
            (score + points_above as f64 - bonus_objective + bonus_offset)
                * bonus_bias
        } else {
            1.0
        }
        - if matches!((N, cell_ind), (5, 13) | (6, 18)) {
            match N {
                5 => 50.0,
                6 => 100.0,
                _ => unreachable!(),
            }
        } else {
            0.0
        }
}

pub fn find_best_cell<const N: u64>(
    scores: &Vec<HashMap<DiceThrow, f64>>,
    throw: &DiceThrow,
    points: &[Option<u64>],
) -> usize {
    let points_above = points.iter().take(6).filter_map(|x| x.as_ref()).sum();
    let ind = (0..scores.len())
        .filter(|&i| points[i].is_none())
        .map(|i| (i, effective_score::<N>(scores, throw, points_above, i)))
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap()
        .0;

    ind
}
