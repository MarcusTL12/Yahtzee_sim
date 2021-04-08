use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    ops::RangeInclusive,
};

use once_cell::sync::Lazy;

use crate::{
    bitfield_array_file::BitfieldArrayFile,
    yahtzee::DiceThrow,
    yahtzee_strats::{make_init_score_tables, re_throw_iters, DiceIter},
};

static DICE_INDEX: Lazy<[HashMap<DiceThrow, usize>; 2]> = Lazy::new(|| {
    [
        DiceIter::new(5).enumerate().map(|(i, d)| (d, i)).collect(),
        DiceIter::new(6).enumerate().map(|(i, d)| (d, i)).collect(),
    ]
});

pub fn get_dice_index<const N: u64>(throw: &DiceThrow) -> &usize {
    DICE_INDEX[match N {
        5 => 0,
        6 => 1,
        _ => unreachable!(),
    }]
    .get(throw)
    .unwrap()
}

static CELLS_INDEX_5: Lazy<(Vec<Vec<[bool; 15]>>, HashMap<[bool; 15], usize>)> =
    Lazy::new(|| {
        let mut all = vec![Vec::new(); 15];

        let mut buf = [false; 15];

        while !buf.iter().all(|&x| x) {
            for i in 0..15 {
                buf[i] = !buf[i];
                if buf[i] {
                    break;
                }
            }

            let amt_free = buf.iter().filter(|&&x| x).count();
            let sum: usize = buf
                .iter()
                .zip((0..15).map(|i| 2usize.pow(i)))
                .map(|(&b, d)| if b { d } else { 0 })
                .sum();

            all[amt_free - 1].push((buf, sum));
        }

        for list in all.iter_mut() {
            list.sort_by_key(|(_, x)| *x);
        }

        let all: Vec<Vec<_>> = all
            .into_iter()
            .map(|list| list.into_iter().map(|(x, _)| x).collect())
            .collect();

        let map = all
            .iter()
            .flat_map(|list| list.iter().enumerate().map(|(i, &x)| (x, i)))
            .collect();

        (all, map)
    });

static CELLS_INDEX_6: Lazy<(Vec<Vec<[bool; 20]>>, HashMap<[bool; 20], usize>)> =
    Lazy::new(|| {
        let mut all = vec![Vec::new(); 20];

        let mut buf = [false; 20];

        while !buf.iter().all(|&x| x) {
            for i in 0..20 {
                buf[i] = !buf[i];
                if buf[i] {
                    break;
                }
            }

            let amt_free = buf.iter().filter(|&&x| x).count();
            let sum: usize = buf
                .iter()
                .zip((0..15).map(|i| 2usize.pow(i)))
                .map(|(&b, d)| if b { d } else { 0 })
                .sum();

            all[amt_free - 1].push((buf, sum));
        }

        for list in all.iter_mut() {
            list.sort_by_key(|(_, x)| *x);
        }

        let all: Vec<Vec<_>> = all
            .into_iter()
            .map(|list| list.into_iter().map(|(x, _)| x).collect())
            .collect();

        let map = all
            .iter()
            .flat_map(|list| list.iter().enumerate().map(|(i, &x)| (x, i)))
            .collect();

        (all, map)
    });

fn get_cell_index<const N: u64>(cells: &[bool]) -> usize {
    if !cells.iter().any(|&x| x) {
        0
    } else {
        match N {
            5 => *CELLS_INDEX_5.1.get(cells).unwrap(),
            6 => *CELLS_INDEX_6.1.get(cells).unwrap(),
            _ => unreachable!(),
        }
    }
}

static STRATS_5_0: Lazy<[Lazy<BitfieldArrayFile<4>>; 15]> = Lazy::new(|| {
    [
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/1_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/2_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/3_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/4_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/5_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/6_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/7_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/8_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/9_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/10_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/11_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/12_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/13_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/14_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/15_0.bfa")),
    ]
});

static STRATS_5_1: Lazy<[Lazy<BitfieldArrayFile<5>>; 15]> = Lazy::new(|| {
    [
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/1_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/2_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/3_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/4_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/5_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/6_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/7_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/8_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/9_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/10_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/11_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/12_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/13_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/14_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/15_1.bfa")),
    ]
});

static STRATS_5_2: Lazy<[Lazy<BitfieldArrayFile<5>>; 15]> = Lazy::new(|| {
    [
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/1_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/2_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/3_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/4_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/5_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/6_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/7_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/8_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/9_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/10_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/11_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/12_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/13_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/14_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/15_2.bfa")),
    ]
});

static STRATS_6_0: Lazy<[Lazy<BitfieldArrayFile<5>>; 20]> = Lazy::new(|| {
    [
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/1_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/2_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/3_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/4_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/5_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/6_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/7_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/8_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/9_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/10_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/11_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/12_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/13_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/14_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/15_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/16_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/17_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/18_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/19_0.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/20_0.bfa")),
    ]
});

static STRATS_6_1: Lazy<[Lazy<BitfieldArrayFile<5>>; 20]> = Lazy::new(|| {
    [
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/1_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/2_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/3_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/4_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/5_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/6_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/7_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/8_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/9_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/10_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/11_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/12_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/13_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/14_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/15_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/16_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/17_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/18_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/19_1.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/20_1.bfa")),
    ]
});

static STRATS_6_2: Lazy<[Lazy<BitfieldArrayFile<5>>; 20]> = Lazy::new(|| {
    [
        Lazy::new(|| BitfieldArrayFile::open("lookup/5/1_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/2_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/3_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/4_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/5_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/6_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/7_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/8_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/9_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/10_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/11_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/12_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/13_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/14_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/15_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/16_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/17_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/18_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/19_2.bfa")),
        Lazy::new(|| BitfieldArrayFile::open("lookup/6/20_2.bfa")),
    ]
});

fn get_score_at_index(scores_file: &mut File, ind: usize) -> f32 {
    let mut buf = [0; 4];
    scores_file.seek(SeekFrom::Start((ind * 4) as u64)).unwrap();
    scores_file.read(&mut buf).unwrap();
    f32::from_le_bytes(buf)
}

#[derive(Clone)]
struct GameState<const N: u64> {
    free_cells: Vec<bool>,
    points_above: u64,
    dice: DiceThrow,
    throws_left: u64,
}

impl<const N: u64> GameState<N> {
    fn amt_free_cells(&self) -> usize {
        self.free_cells.iter().filter(|&&x| x).count()
    }

    fn get_index(&self) -> usize {
        let amt_dice_index = match N {
            5 => 252,
            6 => 462,
            _ => unimplemented!(),
        };

        const AMT_POINTS_INDEX: usize = 121;

        let dice_ind = get_dice_index::<N>(&self.dice);
        let cell_ind = get_cell_index::<N>(&self.free_cells);

        dice_ind
            + (self.points_above as usize + cell_ind * AMT_POINTS_INDEX)
                * amt_dice_index
    }

    fn expected_remaining_score(&self) -> f32 {
        match (self.amt_free_cells(), self.throws_left) {
            (0, _) => 0.0,
            (1, 0) => {
                let free_ind = self
                    .free_cells
                    .iter()
                    .enumerate()
                    .find_map(|(i, &b)| if b { Some(i) } else { None })
                    .unwrap();

                let score = self.dice.cell_score::<N>(free_ind);

                if (free_ind < 6)
                    && (self.points_above + score
                        >= match N {
                            5 => 63,
                            6 => 84,
                            _ => unreachable!(),
                        })
                {
                    (score
                        + match N {
                            5 => 50,
                            6 => 100,
                            _ => unreachable!(),
                        }) as f32
                } else {
                    score as f32
                }
            }
            _ => unimplemented!(),
        }
    }

    fn calc_rethrow_and_score(
        &self,
        scores_file: &mut File,
    ) -> (DiceThrow, f32) {
        let mut new_state = self.clone();

        self.dice
            .clone()
            .into_sub_throw_iter()
            .map(|sub_throw| {
                (
                    sub_throw.clone(),
                    re_throw_iters(&self.dice, &sub_throw)
                        .map(|(throw, prob)| {
                            new_state.dice = throw;

                            let score = get_score_at_index(
                                scores_file,
                                new_state.get_index(),
                            );

                            score * (prob as f32)
                        })
                        .sum::<f32>(),
                )
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    fn calc_cell_and_score(&self, scores_file: &mut File) -> (usize, f32) {
        let mut new_state = self.clone();

        let free_inds: Vec<_> = self
            .free_cells
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b)
            .map(|(i, _)| i)
            .collect();

        free_inds
            .into_iter()
            .map(|i| {
                new_state.free_cells[i] = false;

                new_state.points_above = self.points_above
                    + if i < 6 {
                        self.dice.cell_score::<N>(i)
                    } else {
                        0
                    };

                let score =
                    get_score_at_index(scores_file, new_state.get_index());

                new_state.free_cells[i] = true;

                (i, score)
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}

struct GameStateIter<const N: u64> {
    free_cells: usize,
    current_state: GameState<N>,
    dice_iter: DiceIter,
    points_iter: RangeInclusive<u64>,
    cell_ind: usize,
}

impl<const N: u64> Iterator for GameStateIter<N> {
    type Item = GameState<N>;
    fn next(&mut self) -> Option<Self::Item> {
        let ans = self.current_state.clone();

        if let Some(dice) = self.dice_iter.next() {
            self.current_state.dice = dice;
        } else {
            self.dice_iter = DiceIter::new(N);
            self.current_state.dice = self.dice_iter.next().unwrap();
            if let Some(points) = self.points_iter.next() {
                self.current_state.points_above = points;
            } else {
                self.points_iter = 0..=121;
                self.current_state.points_above =
                    self.points_iter.next().unwrap();
                self.cell_ind += 1;
                match N {
                    5 => {
                        if let Some(x) = CELLS_INDEX_5.0[self.free_cells - 1]
                            .get(self.cell_ind)
                        {
                            for (a, &b) in
                                self.current_state.free_cells.iter_mut().zip(x)
                            {
                                *a = b;
                            }
                        } else {
                            return None;
                        }
                    }
                    6 => {
                        if let Some(x) = CELLS_INDEX_6.0[self.free_cells - 1]
                            .get(self.cell_ind)
                        {
                            for (a, &b) in
                                self.current_state.free_cells.iter_mut().zip(x)
                            {
                                *a = b;
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        Some(ans)
    }
}

impl<const N: u64> GameStateIter<N> {
    fn new(free_cells: usize) -> Self {
        let mut dice_iter = DiceIter::new(N);
        let mut points_iter = 0..=121;
        let cells = match N {
            5 => CELLS_INDEX_5.0[free_cells - 1][0].iter().cloned().collect(),
            6 => CELLS_INDEX_6.0[free_cells - 1][0].iter().cloned().collect(),
            _ => unreachable!(),
        };
        let state = GameState {
            free_cells: cells,
            points_above: points_iter.next().unwrap(),
            dice: dice_iter.next().unwrap(),
            throws_left: 0,
        };
        Self {
            free_cells,
            current_state: state,
            dice_iter,
            points_iter,
            cell_ind: 0,
        }
    }
}

fn make_first_scores_file<const N: u64>() {
    let mut buf = Vec::new();
    for bytes in GameStateIter::<N>::new(1)
        .map(|state| state.expected_remaining_score().to_le_bytes())
    {
        buf.extend(bytes.iter().cloned());
    }
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("lookup/scores/1_0.bin")
        .unwrap();

    file.write_all(&buf).unwrap();
}

fn rethrow_bits<const N: usize>(
    orig_dice: DiceThrow,
    rethrow: DiceThrow,
) -> [bool; N] {
    let mut bits = [false; N];

    let mut ind = 0;

    for i in 1..=6 {
        for j in 0..rethrow[i] {
            bits[(ind + j) as usize] = true;
        }
        ind += orig_dice[i];
    }

    bits
}

fn make_scores_and_rethrows_from_prev<const N: usize>(
    free_cells: usize,
    throws_left: usize,
    scores_file: &mut File,
) {
    let mut new_scores = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(format!("lookup/scores/{}_{}.bin", free_cells, throws_left))
            .unwrap(),
    );
    let mut new_strats = BitfieldArrayFile::<N>::open(&format!(
        "lookup/{}/{}_{}.bfa",
        N, free_cells, throws_left
    ));

    match N {
        5 => {
            for state in GameStateIter::<5>::new(free_cells) {
                let (rethrow, score) =
                    state.calc_rethrow_and_score(scores_file);
                new_strats.push(rethrow_bits(state.dice, rethrow));
                new_scores.write_all(&score.to_le_bytes()).unwrap();
            }
        }
        6 => {
            for state in GameStateIter::<6>::new(free_cells) {
                let (rethrow, score) =
                    state.calc_rethrow_and_score(scores_file);
                new_strats.push(rethrow_bits(state.dice, rethrow));
                new_scores.write_all(&score.to_le_bytes()).unwrap();
            }
        }
        _ => unreachable!(),
    }
}

pub fn test() {
    // make_first_scores_file::<5>();

    make_scores_and_rethrows_from_prev::<5>(
        1,
        1,
        &mut File::open("lookup/scores/1_0.bin").unwrap(),
    );
}
