use std::{
    collections::{HashMap, VecDeque},
    fs::{create_dir_all, read_to_string, File, OpenOptions},
    io::{BufWriter, Read, Write},
    path::Path,
    thread::{sleep, spawn},
    time::{Duration, Instant},
};

use crate::{
    bitfield_array_file::{self, BitfieldArrayFile},
    yahtzee::DiceThrow,
    yahtzee_strats::{re_throw_iters, DiceIter},
};

use num_integer::binomial;

use once_cell::sync::Lazy;

static NUM_CPUS: Lazy<usize> = Lazy::new(|| {
    if Path::new("cpu_count.txt").exists() {
        read_to_string("cpu_count.txt").unwrap().parse().unwrap()
    } else {
        num_cpus::get()
    }
});

static LOOKUP_PATH: Lazy<String> =
    Lazy::new(|| read_to_string("lookup_path.txt").unwrap());

static CELLS: Lazy<Vec<(Vec<Vec<Vec<bool>>>, Vec<HashMap<Vec<bool>, usize>>)>> =
    Lazy::new(|| {
        [15, 20]
            .iter()
            .map(|&n| {
                let mut buf = vec![false; n];

                let mut cells = vec![Vec::new(); n + 1];

                loop {
                    let amt_free = buf.iter().filter(|&&b| b).count();

                    cells[amt_free].push(buf.clone());

                    for b in buf.iter_mut() {
                        *b = !*b;

                        if *b {
                            break;
                        }
                    }

                    if !buf.iter().any(|&b| b) {
                        break;
                    }
                }

                let maps = cells
                    .iter()
                    .map(|v| {
                        v.iter()
                            .enumerate()
                            .map(|(i, v)| (v.clone(), i))
                            .collect()
                    })
                    .collect();

                (cells, maps)
            })
            .collect()
    });

static DICE_INDEX: Lazy<Vec<HashMap<DiceThrow, usize>>> = Lazy::new(|| {
    [5, 6]
        .iter()
        .map(|&n| DiceIter::new(n).enumerate().map(|(i, d)| (d, i)).collect())
        .collect()
});

fn points_for_single_cell<const N: u64>(
    cell_ind: usize,
    dice: DiceThrow,
    points_above: u64,
) -> u64 {
    let score = dice.cell_score::<N>(cell_ind);

    let effective_score = if (cell_ind < 6)
        && (points_above + score
            >= match N {
                5 => 63,
                6 => 84,
                _ => unreachable!(),
            }) {
        score
            + match N {
                5 => 50,
                6 => 100,
                _ => unreachable!(),
            }
    } else {
        score
    };

    effective_score
}

fn amt_dice_index<const N: u64>() -> usize {
    match N {
        5 => 252,
        6 => 462,
        _ => unreachable!(),
    }
}

fn amt_cells<const N: u64>() -> usize {
    match N {
        5 => 15,
        6 => 20,
        _ => unreachable!(),
    }
}

fn amt_cell_ind<const N: u64>(amt_free: usize) -> usize {
    binomial(amt_cells::<N>(), amt_free)
}

fn amt_points_above<const N: u64>() -> usize {
    match N {
        5 => 64,
        6 => 85,
        _ => unreachable!(),
    }
}

fn n_to_ind<const N: u64>() -> usize {
    match N {
        5 => 0,
        6 => 1,
        _ => unreachable!(),
    }
}

fn make_init_scores<const N: u64, const BITS: usize>() {
    let p = Path::new(&*LOOKUP_PATH);

    let strats_path = p.join(format!("{}/strats/1_0/", N));
    create_dir_all(&strats_path).unwrap();

    let scores_path = p.join(format!("{}/scores/1_0/", N));
    create_dir_all(&scores_path).unwrap();

    for points_above in 0..amt_points_above::<N>() {
        let mut scores_file = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(scores_path.join(format!("{}.bin", points_above)))
                .unwrap(),
        );

        let mut strats_file = BitfieldArrayFile::<BITS>::open(
            strats_path.join(format!("{}.bin", points_above)),
        );

        for cell_ind in 0..amt_cells::<N>() {
            for dice in DiceIter::new(N) {
                let score = points_for_single_cell::<N>(
                    cell_ind,
                    dice,
                    points_above as u64,
                );

                strats_file.push(num_to_bits(cell_ind));

                scores_file
                    .write_all(&(score as f32).to_le_bytes())
                    .unwrap();
            }
        }

        strats_file.flush();
        scores_file.flush().unwrap();
    }
}

fn load_scores<const N: u64>(
    free_cells: usize,
    throws_left: usize,
    points_above: u64,
) -> Vec<f32> {
    let mut file = File::open(Path::new(&*LOOKUP_PATH).join(format!(
        "{}/scores/{}_{}/{}.bin",
        N, free_cells, throws_left, points_above
    )))
    .unwrap();

    let chunk_size = 4 * amt_cell_ind::<N>(free_cells) * amt_dice_index::<N>();

    let mut buf = vec![0; chunk_size];

    file.read_exact(&mut buf).unwrap();

    let scores = buf
        .chunks(4)
        .map(|bytes| {
            f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        })
        .collect();

    scores
}

fn get_index<const N: u64>(dice: &DiceThrow, cell_ind: usize) -> usize {
    let dice_ind = DICE_INDEX[match N {
        5 => 0,
        6 => 1,
        _ => unreachable!(),
    }]
    .get(dice)
    .unwrap();

    dice_ind + cell_ind * amt_dice_index::<N>()
}

fn rethrow_bits<const BITS: usize>(
    orig_dice: &DiceThrow,
    rethrow: &DiceThrow,
) -> [bool; BITS] {
    let mut bits = [false; BITS];

    let mut ind = 0;

    for i in 1..=6 {
        for j in 0..rethrow[i] {
            bits[(ind + j) as usize] = true;
        }
        ind += orig_dice[i];
    }

    bits
}

fn make_rethrows_and_scores<const N: u64, const BITS: usize>(
    free_cells: usize,
    throws_left: usize,
) {
    println!(
        "Computing rethrows for {} free cells and {} throws left:",
        free_cells, throws_left
    );

    let supertimer = Instant::now();

    let scores_path = Path::new(&*LOOKUP_PATH)
        .join(format!("{}/scores/{}_{}/", N, free_cells, throws_left));

    let strats_path = Path::new(&*LOOKUP_PATH)
        .join(format!("{}/strats/{}_{}/", N, free_cells, throws_left));

    create_dir_all(&scores_path).unwrap();
    create_dir_all(&strats_path).unwrap();

    let n = amt_cell_ind::<N>(free_cells)
        * amt_points_above::<N>()
        * amt_dice_index::<N>();

    let (progress_s, progress_r) = crossbeam_channel::unbounded();

    let (pause_s, pause_r) = crossbeam_channel::unbounded();
    let (wake_s, wake_r) = crossbeam_channel::unbounded();

    let (index_s, index_r) = crossbeam_channel::unbounded();

    for points_above in 0..amt_points_above::<N>() {
        index_s.send(points_above).unwrap();
    }

    let (done_s, done_r) = crossbeam_channel::unbounded();

    let progress_handle = spawn(move || {
        let mut i = 0;
        let mut timer = Instant::now();
        let mut paused = 0;
        let mut to_be_paused;
        let mut amt_done = 0;
        const SPEED_BUFFER_SIZE: usize = 60;
        let printerval = Duration::from_secs(1);
        let mut speed_buffer = VecDeque::new();
        let mut threads_buffer = VecDeque::new();
        let mut current_counter;
        while i < n {
            current_counter = progress_r.try_iter().sum::<usize>();
            speed_buffer.push_back(current_counter);
            if speed_buffer.len() > SPEED_BUFFER_SIZE {
                speed_buffer.pop_front();
            }

            i += current_counter;

            threads_buffer.push_back(*NUM_CPUS - paused - amt_done);

            if threads_buffer.len() > SPEED_BUFFER_SIZE {
                threads_buffer.pop_front();
            }

            if Path::new("hold_up").exists() {
                if let Ok(amt) =
                    read_to_string("hold_up").unwrap().parse::<usize>()
                {
                    if amt_done > amt {
                        to_be_paused = 0;
                    } else {
                        to_be_paused = amt - amt_done;
                    }
                } else {
                    to_be_paused = *NUM_CPUS;
                }
            } else {
                to_be_paused = 0;
            }

            if paused < to_be_paused {
                for _ in 0..(to_be_paused - paused) {
                    pause_s.send(()).unwrap();
                }
            } else if paused > to_be_paused {
                for _ in 0..(paused - to_be_paused) {
                    wake_s.send(()).unwrap();
                }
            }
            paused = to_be_paused;

            for _ in done_r.try_iter() {
                amt_done += 1;
            }

            if paused != *NUM_CPUS {
                let speed = speed_buffer.iter().sum::<usize>() as f32
                    / (printerval.as_secs_f32() * speed_buffer.len() as f32);

                let speed_per_thread = if threads_buffer.iter().all(|&x| x > 0) {
                    speed_buffer
                        .iter()
                        .zip(threads_buffer.iter())
                        .map(|(&s, &t)| s as f32 / t as f32)
                        .sum::<f32>()
                        / (printerval.as_secs_f32() * speed_buffer.len() as f32)
                } else {
                    0.0
                };

                println!(
                    "{} / {} = {:.2}%       speed = {:.1}       speed per thread = {:.1}",
                    i,
                    n,
                    (i as f32) / (n as f32) * 100.0,
                    speed,
                    speed_per_thread
                );
            }

            if timer.elapsed() < printerval {
                sleep(printerval - timer.elapsed());
                timer += printerval;
            }
        }
    });

    let handles: Vec<_> = (0..*NUM_CPUS)
        .map(|i| {
            let scores_path = scores_path.clone();
            let strats_path = strats_path.clone();
            let progress_s = progress_s.clone();
            let pause_r = pause_r.clone();
            let wake_r = wake_r.clone();
            let mut paused = false;
            let index_r = index_r.clone();
            let done_s = done_s.clone();
            spawn(move || {
                let mut count = 0;
                let mut timer = Instant::now();
                while let Ok(points_above) = index_r.try_recv() {
                    let mut scores_file = BufWriter::with_capacity(
                        1024 * 1024,
                        OpenOptions::new()
                            .create(true)
                            .truncate(true)
                            .write(true)
                            .open(
                                scores_path
                                    .join(format!("{}.bin", points_above)),
                            )
                            .unwrap(),
                    );
                    let mut strats_file = BitfieldArrayFile::<BITS>::open(
                        strats_path.join(format!("{}.bin", points_above)),
                    );

                    let scores = load_scores::<N>(
                        free_cells,
                        throws_left - 1,
                        points_above as u64,
                    );
                    for cell_ind in 0..amt_cell_ind::<N>(free_cells) {
                        for dice in DiceIter::new(N) {
                            let (sub_throw, score) = dice
                                .clone()
                                .into_sub_throw_iter()
                                .map(|sub_throw| {
                                    (
                                        sub_throw.clone(),
                                        re_throw_iters(&dice, &sub_throw)
                                            .map(|(throw, prob)| {
                                                prob as f32
                                                    * scores[get_index::<N>(
                                                        &throw, cell_ind,
                                                    )]
                                            })
                                            .sum::<f32>(),
                                    )
                                })
                                .max_by(|(_, a), (_, b)| {
                                    a.partial_cmp(b).unwrap()
                                })
                                .unwrap();
                            scores_file
                                .write_all(&score.to_le_bytes())
                                .unwrap();
                            strats_file.push(rethrow_bits(&dice, &sub_throw));

                            count += 1;

                            if timer.elapsed() >= Duration::from_secs(1) {
                                timer += Duration::from_secs(1);
                                progress_s.send(count).unwrap();
                                count = 0;
                                if let Ok(()) = pause_r.try_recv() {
                                    paused = true;
                                }
                            }

                            while paused {
                                sleep(Duration::from_secs(1));
                                if let Ok(()) = wake_r.try_recv() {
                                    paused = false;
                                }
                            }
                        }
                    }
                    scores_file.flush().unwrap();
                    strats_file.flush();
                }

                progress_s.send(count).unwrap();
                done_s.send(i).unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    progress_handle.join().unwrap();

    println!("took {:?}\n", supertimer.elapsed());
}

fn num_to_bits<const BITS: usize>(n: usize) -> [bool; BITS] {
    let mut buf = [false; BITS];

    for i in 0..BITS {
        buf[i] = ((n >> i) & 1) != 0;
    }

    buf
}

fn make_cell_choice_and_scores<const N: u64, const BITS: usize>(
    free_cells: usize,
) {
    println!("Computing cell choice for {} free cells:", free_cells);

    let supertimer = Instant::now();

    let scores_path = Path::new(&*LOOKUP_PATH)
        .join(format!("{}/scores/{}_{}/", N, free_cells, 0));

    let strats_path = Path::new(&*LOOKUP_PATH)
        .join(format!("{}/strats/{}_{}/", N, free_cells, 0));

    create_dir_all(&scores_path).unwrap();
    create_dir_all(&strats_path).unwrap();

    let mut scores_buf = Vec::new();

    let mut highets_points_in_buffer = match N {
        5 => 30,
        6 => 36,
        _ => unreachable!(),
    };

    for points_above in 0..=match N {
        5 => 30,
        6 => 36,
        _ => unimplemented!(),
    } {
        scores_buf.push(load_scores::<N>(free_cells - 1, 2, points_above));
    }

    let mut i = 0;
    let n = amt_cell_ind::<N>(free_cells)
        * amt_points_above::<N>()
        * amt_dice_index::<N>();

    let mut timer = Instant::now();

    for points_above in 0..amt_points_above::<N>() {
        if Path::new("hold_up").exists() {
            if let Err(_) = read_to_string("hold_up").unwrap().parse::<usize>()
            {
                sleep(Duration::from_secs(10));
                timer = Instant::now();
            }
        }

        let mut scores_file = BufWriter::with_capacity(
            1024 * 1024,
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(scores_path.join(format!("{}.bin", points_above)))
                .unwrap(),
        );
        let mut strats_file = BitfieldArrayFile::<BITS>::open(
            strats_path.join(format!("{}.bin", points_above)),
        );

        let mut free_inds = Vec::new();
        for cell_ind in 0..amt_cell_ind::<N>(free_cells) {
            for dice in DiceIter::new(N) {
                let mut cells =
                    CELLS[n_to_ind::<N>()].0[free_cells][cell_ind].clone();

                free_inds.clear();
                free_inds.extend((0..amt_cells::<N>()).filter(|&i| cells[i]));

                let (best_ind, score) = free_inds
                    .iter()
                    .map(|&i| {
                        cells[i] = false;

                        let &n_cell_ind = CELLS[n_to_ind::<N>()].1
                            [free_cells - 1]
                            .get(&cells)
                            .unwrap();

                        let n_ind = get_index::<N>(&dice, n_cell_ind);

                        let additional_points = dice.cell_score::<N>(i);

                        let mut points_offset =
                            if i < 6 { additional_points as usize } else { 0 };

                        let mut bonus = 0.0;

                        if points_offset + points_above
                            >= amt_points_above::<N>() - 1
                        {
                            points_offset = scores_buf.len() - 1;
                            bonus = match N {
                                5 => 50.0,
                                6 => 100.0,
                                _ => unreachable!(),
                            }
                        }

                        let score = scores_buf[points_offset][n_ind]
                            + additional_points as f32
                            + bonus;

                        cells[i] = true;

                        (i, score)
                    })
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .unwrap();

                scores_file.write_all(&score.to_le_bytes()).unwrap();
                strats_file.push(num_to_bits::<BITS>(best_ind));

                i += 1;

                if timer.elapsed() >= Duration::from_secs(1) {
                    timer += Duration::from_secs(1);

                    println!(
                        "{} / {} = {:.2}%",
                        i,
                        n,
                        (i as f32) / (n as f32) * 100.0
                    );
                }
            }
        }

        scores_buf.remove(0);
        if highets_points_in_buffer + 1 < amt_points_above::<N>() {
            highets_points_in_buffer += 1;
            scores_buf.push(load_scores::<N>(
                free_cells - 1,
                2,
                highets_points_in_buffer as u64,
            ));
        }

        scores_file.flush().unwrap();
        strats_file.flush();
    }

    println!("took {:?}\n", supertimer.elapsed());
}

pub fn init(n: &str) {
    match n {
        "5" => make_init_scores::<5, 4>(),
        "6" => make_init_scores::<6, 5>(),
        _ => println!("invalid number of dice"),
    }
}

pub fn resume_calcs_5(mut free_cells: usize, throws_left: usize) {
    let timer = Instant::now();
    if throws_left == 1 {
        make_rethrows_and_scores::<5, 5>(free_cells, 1);
    }
    if Path::new("wrap_up").exists() {
        return;
    }
    if throws_left > 0 {
        make_rethrows_and_scores::<5, 5>(free_cells, 2);
        free_cells += 1;
    }
    if Path::new("wrap_up").exists() {
        return;
    }

    for free_cells in free_cells..=15 {
        make_cell_choice_and_scores::<5, 4>(free_cells);
        if Path::new("wrap_up").exists() {
            break;
        }
        make_rethrows_and_scores::<5, 5>(free_cells, 1);
        if Path::new("wrap_up").exists() {
            break;
        }
        make_rethrows_and_scores::<5, 5>(free_cells, 2);
        if Path::new("wrap_up").exists() {
            break;
        }
    }

    println!("Total time: {:?}", timer.elapsed());
}

pub fn resume_calcs_6(mut free_cells: usize, throws_left: usize) {
    if throws_left == 1 {
        make_rethrows_and_scores::<6, 6>(free_cells, 1);
    }
    if Path::new("wrap_up").exists() {
        return;
    }
    if throws_left > 0 {
        make_rethrows_and_scores::<6, 6>(free_cells, 2);
        free_cells += 1;
    }
    if Path::new("wrap_up").exists() {
        return;
    }

    for free_cells in free_cells..=15 {
        make_cell_choice_and_scores::<6, 5>(free_cells);
        if Path::new("wrap_up").exists() {
            break;
        }
        make_rethrows_and_scores::<6, 6>(free_cells, 1);
        if Path::new("wrap_up").exists() {
            break;
        }
        make_rethrows_and_scores::<6, 6>(free_cells, 2);
        if Path::new("wrap_up").exists() {
            break;
        }
    }
}

fn get_dice_from_bits(orig_dice: &DiceThrow, bits: &[bool]) -> DiceThrow {
    let mut dice = DiceThrow::from([0; 6]);

    for (i, n) in orig_dice.into_ordered_dice().enumerate() {
        if bits[i] {
            dice[n] += 1;
        }
    }

    dice
}

pub fn get_rethrow_strat<const N: u64>(
    cells: &[bool],
    dice: &DiceThrow,
    throws_left: usize,
    points_above: u64,
) -> DiceThrow {
    let free_cells = cells.iter().filter(|&&b| b).count();
    let &cell_ind = CELLS[n_to_ind::<N>()].1[free_cells].get(cells).unwrap();
    let ind = get_index::<N>(dice, cell_ind);
    let path = Path::new(&*LOOKUP_PATH).join(format!(
        "{}/strats/{}_{}/{}.bin",
        N,
        free_cells,
        throws_left,
        if points_above as usize + 1 >= amt_points_above::<N>() {
            amt_points_above::<N>() - 1
        } else {
            points_above as usize
        }
    ));

    let rethrow = match N {
        5 => get_dice_from_bits(
            dice,
            &bitfield_array_file::get_bits::<_, 5>(&path, ind),
        ),
        6 => get_dice_from_bits(
            dice,
            &bitfield_array_file::get_bits::<_, 6>(path, ind),
        ),
        _ => unreachable!(),
    };

    rethrow
}

fn get_ind_from_bits(bits: &[bool]) -> usize {
    let mut bitval = 1;
    let mut acc = 0;

    for &bit in bits {
        if bit {
            acc += bitval
        }
        bitval <<= 1;
    }

    acc
}

pub fn get_cell_strat<const N: u64>(
    cells: &[bool],
    dice: &DiceThrow,
    points_above: u64,
) -> usize {
    let free_cells = cells.iter().filter(|&&b| b).count();
    let &cell_ind = CELLS[n_to_ind::<N>()].1[free_cells].get(cells).unwrap();
    let ind = get_index::<N>(dice, cell_ind);
    let path = Path::new(&*LOOKUP_PATH).join(format!(
        "{}/strats/{}_0/{}.bin",
        N,
        free_cells,
        if points_above as usize + 1 >= amt_points_above::<N>() {
            amt_points_above::<N>() - 1
        } else {
            points_above as usize
        }
    ));

    match N {
        5 => get_ind_from_bits(&bitfield_array_file::get_bits::<_, 4>(
            &path, ind,
        )),
        6 => get_ind_from_bits(&bitfield_array_file::get_bits::<_, 5>(
            &path, ind,
        )),
        _ => unimplemented!(),
    }
}

pub fn test(_commands: &[&str]) {}
