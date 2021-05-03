use std::time::{Duration, Instant};

use crate::{
    yahtzee_free_strats::{get_cell_strat, get_rethrow_strat},
    yahtzee_guide::display_points,
    yahtzee_strats::new_throw,
    DiceThrow,
};

fn simulate_game<const N: u64>(
    points: &mut [Option<u64>],
    free_cells: &mut [bool],
) {
    for _ in 0..match N {
        5 => 15,
        6 => 20,
        _ => unreachable!(),
    } {
        let mut dice = DiceThrow::throw(N as usize);

        let points_above =
            points.iter().take(6).filter_map(|x| x.as_ref()).sum();

        for &throws_left in &[2, 1] {
            let rethrow = get_rethrow_strat::<N>(
                &free_cells,
                &dice,
                throws_left,
                points_above,
            );

            let th = DiceThrow::throw(rethrow.amt_dice() as usize);

            dice = new_throw(&dice, &rethrow, &th);
        }

        let ind = get_cell_strat::<N>(&free_cells, &dice, points_above);

        let score = dice.cell_score::<N>(ind);

        points[ind] = Some(score);
        free_cells[ind] = false;
    }
}

pub fn simulate_single_game<const N: u64>() {
    let cells = match N {
        5 => 15,
        6 => 20,
        _ => unreachable!(),
    };
    let mut points = vec![None; cells];
    let mut free_cells = vec![true; cells];

    simulate_game::<N>(&mut points, &mut free_cells);

    display_points::<_, N>(&points, None, None);
}

pub fn simulate_multiple<const N: u64>(n: usize) {
    let cells = match N {
        5 => 15,
        6 => 20,
        _ => unreachable!(),
    };
    let bonus_objective = match N {
        5 => 63,
        6 => 84,
        _ => unreachable!(),
    };
    let bonus_amt = match N {
        5 => 50,
        6 => 100,
        _ => unreachable!(),
    };
    let mut points = vec![None; cells];
    let mut free_cells = vec![true; cells];

    let mut averages = vec![Some(0.0); cells];
    let mut avg_bonus = 0.0;
    let mut avg_sum = 0.0;

    let mut timer = Instant::now();

    for i in 0..n {
        if timer.elapsed() > Duration::from_secs(1) {
            println!("{} / {}", i, n);
            timer += Duration::from_secs(1);
        }

        for x in points.iter_mut() {
            *x = None;
        }
        for x in free_cells.iter_mut() {
            *x = true;
        }
        simulate_game::<N>(&mut points, &mut free_cells);

        let bonus = if points
            .iter()
            .take(6)
            .filter_map(|x| x.as_ref())
            .sum::<u64>()
            >= bonus_objective
        {
            bonus_amt
        } else {
            0
        };

        avg_bonus += bonus as f32;

        avg_sum += (bonus as f32)
            + points
                .iter()
                .filter_map(|x| x.and_then(|x| Some(x as f32)))
                .sum::<f32>();

        for (a, x) in averages.iter_mut().zip(points.iter()) {
            if let &Some(x) = x {
                if let Some(a) = a {
                    *a += x as f32;
                }
            }
        }
    }

    for x in averages.iter_mut() {
        if let Some(x) = x {
            *x /= n as f32;
        }
    }

    avg_bonus /= n as f32;
    avg_sum /= n as f32;

    display_points::<_, N>(&averages, Some(avg_bonus), Some(avg_sum));
}
