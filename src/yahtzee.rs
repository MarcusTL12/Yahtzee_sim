use std::{
    fmt::{Display, Error, Formatter},
    ops::{Index, IndexMut},
};

use rand::prelude::*;

#[derive(Debug)]
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

impl DiceThrow {
    fn new() -> Self {
        Self { dice: [0; 6] }
    }

    pub fn roll(n: usize) -> Self {
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

    // pub fn pairs<const N: usize>(&self) -> u64 {
    //     let mut amt = 0;
    //     let mut score = 0;

    //     for i in (1..=6).rev() {
    //         if self[i] >= 2 {
    //             amt += 1;
    //             score += 2 * i;
    //         }

    //         if amt == N {
    //             return score;
    //         }
    //     }

    //     0
    // }

    pub fn pairs<const N: usize>(&self) -> u64 {
        (1..=6)
            .rev()
            .filter_map(|i| if self[i] >= 2 { Some(i * 2) } else { None })
            .take(N)
            .sum()
    }

    // pub fn n_of_a_kind<const N: u64>(&self) -> u64 {
    //     for i in (1..=6).rev() {
    //         if self[i] >= N {
    //             return i * N;
    //         }
    //     }

    //     0
    // }

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
}
