use std::{
    fmt::{Display, Error, Formatter},
    ops::{Index, IndexMut},
};

use serde::{Deserialize, Serialize};

use rand::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiceThrow {
    dice: [u64; 6],
}

impl Index<u64> for DiceThrow {
    type Output = u64;
    fn index(&self, index: u64) -> &Self::Output {
        &self.dice[index as usize - 1]
    }
}

impl IndexMut<u64> for DiceThrow {
    fn index_mut(&mut self, index: u64) -> &mut Self::Output {
        &mut self.dice[index as usize - 1]
    }
}

impl Display for DiceThrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        const EYES: &[[char; 9]] = &[
            [' ', ' ', ' ', ' ', '●', ' ', ' ', ' ', ' '],
            ['●', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '●'],
            ['●', ' ', ' ', ' ', '●', ' ', ' ', ' ', '●'],
            ['●', ' ', '●', ' ', ' ', ' ', '●', ' ', '●'],
            ['●', ' ', '●', ' ', '●', ' ', '●', ' ', '●'],
            ['●', ' ', '●', '●', ' ', '●', '●', ' ', '●'],
        ];

        for i in 1..=6 {
            for _ in 0..self[i] {
                write!(f, "┏━━━━━━━┓")?;
            }
        }
        writeln!(f, "")?;

        for i in (0..9).step_by(3) {
            for j in 1..=6 {
                let eyes = &EYES[j as usize - 1];
                for _ in 0..self[j] {
                    write!(
                        f,
                        "┃ {} {} {} ┃",
                        eyes[i],
                        eyes[i + 1],
                        eyes[i + 2]
                    )?;
                }
            }
            writeln!(f, "")?;
        }

        for i in 1..=6 {
            for _ in 0..self[i] {
                write!(f, "┗━━━━━━━┛")?;
            }
        }

        Ok(())
    }
}

impl From<[u64; 6]> for DiceThrow {
    fn from(dice: [u64; 6]) -> Self {
        Self { dice }
    }
}

impl DiceThrow {
    fn new() -> Self {
        Self { dice: [0; 6] }
    }

    pub fn throw(n: usize) -> Self {
        let mut dice_throw = Self::new();

        let mut rng = rand::thread_rng();

        for _ in 0..n {
            let eyes = rng.gen_range(1..=6);

            dice_throw[eyes] += 1;
        }

        dice_throw
    }

    pub fn ammount_of<const N: u64>(&self) -> u64 {
        self[N] * N
    }

    pub fn pairs<const N: usize>(&self) -> u64 {
        let (score, amt) = (1..=6)
            .rev()
            .filter_map(|i| if self[i] >= 2 { Some(i * 2) } else { None })
            .take(N)
            .fold((0, 0), |(a, amt), x| (a + x, amt + 1));

        if amt == N {
            score
        } else {
            0
        }
    }

    pub fn n_of_a_kind<const N: u64>(&self) -> u64 {
        (1..=6)
            .rev()
            .find_map(|i| if self[i] >= N { Some(i * N) } else { None })
            .unwrap_or(0)
    }

    pub fn straight<const A: u64, const B: u64>(&self) -> u64 {
        if (A..=B).all(|i| self[i] >= 1) {
            (A..=B).sum()
        } else {
            0
        }
    }

    pub fn building<const A: u64, const B: u64>(&self) -> u64 {
        if let Some(a) = (1..=6).rev().filter(|&i| self[i] >= A).next() {
            if let Some(b) = (1..=6)
                .rev()
                .filter(|&i| i != a)
                .filter(|&i| self[i] >= B)
                .next()
            {
                A * a + B * b
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn chance(&self) -> u64 {
        (1..=6).map(|i| self[i] * i).sum()
    }

    pub fn yahtzee(&self) -> u64 {
        let amt_dice: u64 = (1..=6).map(|i| self[i]).sum();

        if (1..=6).any(|i| self[i] == amt_dice) {
            match amt_dice {
                5 => 50,
                6 => 100,
                _ => unreachable!(),
            }
        } else {
            0
        }
    }

    pub fn cell_score<const N: u64>(&self, cell_ind: usize) -> u64 {
        match (N, cell_ind) {
            (_, 0) => self.ammount_of::<1>(),
            (_, 1) => self.ammount_of::<2>(),
            (_, 2) => self.ammount_of::<3>(),
            (_, 3) => self.ammount_of::<4>(),
            (_, 4) => self.ammount_of::<5>(),
            (_, 5) => self.ammount_of::<6>(),
            (_, 6) => self.pairs::<1>(),
            (_, 7) => self.pairs::<2>(),
            (6, 8) => self.pairs::<3>(),
            (5, 8) | (6, 9) => self.n_of_a_kind::<3>(),
            (5, 9) | (6, 10) => self.n_of_a_kind::<4>(),
            (6, 11) => self.n_of_a_kind::<5>(),
            (5, 10) | (6, 12) => self.straight::<1, 5>(),
            (5, 11) | (6, 13) => self.straight::<2, 6>(),
            (6, 14) => self.straight::<1, 6>(),
            (5, 12) | (6, 15) => self.building::<3, 2>(),
            (6, 16) => self.building::<3, 3>(),
            (6, 17) => self.building::<4, 2>(),
            (5, 13) | (6, 18) => self.chance(),
            (5, 14) | (6, 19) => self.yahtzee(),
            _ => unreachable!(),
        }
    }

    pub fn test_all(&self) {
        println!("1: {}", self.ammount_of::<1>());
        println!("2: {}", self.ammount_of::<2>());
        println!("3: {}", self.ammount_of::<3>());
        println!("4: {}", self.ammount_of::<4>());
        println!("5: {}", self.ammount_of::<5>());
        println!("6: {}", self.ammount_of::<6>());
        println!("--------");
        println!("1 pair: {}", self.pairs::<1>());
        println!("2 pair: {}", self.pairs::<2>());
        println!("3 pair: {}", self.pairs::<3>());
        println!("3 of a kind: {}", self.n_of_a_kind::<3>());
        println!("4 of a kind: {}", self.n_of_a_kind::<4>());
        println!("5 of a kind: {}", self.n_of_a_kind::<5>());
        println!("Small straight: {}", self.straight::<1, 5>());
        println!("Large straight: {}", self.straight::<2, 6>());
        println!("Full  straight: {}", self.straight::<1, 6>());
        println!("Hut:   {}", self.building::<3, 2>());
        println!("House: {}", self.building::<3, 3>());
        println!("Tower: {}", self.building::<4, 2>());
        println!("Chance: {}", self.chance());
        println!("Yahtzee: {}", self.yahtzee());
    }

    pub fn into_sub_throw_iter(self) -> SubThrowIter {
        SubThrowIter::from(self)
    }

    pub fn amt_dice(&self) -> u64 {
        (1..=6).map(|i| self[i]).sum()
    }

    pub fn probability(&self) -> f64 {
        let amt_dice = self.amt_dice();

        let tot = 6u64.pow(amt_dice as u32);

        let perms: u64 = factorial(amt_dice);

        let dup_perms: u64 = (1..=6).map(|i| factorial(self[i])).product();

        let actual_perms = perms / dup_perms;

        (actual_perms as f64) / (tot as f64)
    }

    pub fn into_ordered_dice<'a>(&'a self) -> impl Iterator<Item = u64> + 'a {
        self.dice
            .iter()
            .enumerate()
            .flat_map(|(i, &amt)| (0..amt).map(move |_| (i as u64) + 1))
    }
}

fn factorial(n: u64) -> u64 {
    (2..=n).product()
}

pub struct SubThrowIter {
    done: bool,
    dice: DiceThrow,
    sub_throw: DiceThrow,
}

impl From<DiceThrow> for SubThrowIter {
    fn from(dice: DiceThrow) -> Self {
        SubThrowIter {
            done: false,
            dice,
            sub_throw: DiceThrow::from([0; 6]),
        }
    }
}

impl Iterator for SubThrowIter {
    type Item = DiceThrow;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let ans = Some(self.sub_throw.clone());
            for i in 1..=6 {
                if self.sub_throw[i] < self.dice[i] {
                    self.sub_throw[i] += 1;
                    break;
                } else {
                    self.sub_throw[i] = 0;
                    if i == 6 {
                        self.done = true;
                    }
                }
            }
            ans
        }
    }
}
