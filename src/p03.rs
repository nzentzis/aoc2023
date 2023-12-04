use anyhow::{anyhow, Result};
use std::collections::HashSet;
use crate::grid::Grid;

#[derive(Copy, Clone, Debug)]
enum Cell {
    Empty,
    Symbol(char),
    Digit(u8),
}

impl TryFrom<char> for Cell {
    type Error = anyhow::Error;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            v if v.is_ascii_digit() => Self::Digit(v as u8 - b'0'),
            '.'                     => Self::Empty,
            c                       => Self::Symbol(c),
        })
    }
}

/// Mapping of the part numbers on a given symbol grid
///
/// This handles merging multi-digit part numbers from adjacent cells, and contains a mapping of
/// which part number ID is stored in each grid cell.
struct NumberMap {
    numbers: Vec<u32>,
    number_ids: Grid<Option<usize>>,
}

impl NumberMap {
    fn from_grid(grid: &Input) -> Self {
        enum State {
            Idle,
            Number {
                accum: u32,
                ident: usize,
            },
        }

        // find contiguous numbers by running a state machine over each line
        let mut numbers = Vec::new();
        let mut number_ids = Grid::filled_like(&grid, None);

        for row_idx in 0..grid.height() {
            let mut state = State::Idle;
            for cell in grid.row_iter(row_idx) {
                match (&mut state, *cell) {
                    (State::Idle, Cell::Digit(n)) => {
                        state = State::Number { accum: n as u32, ident: numbers.len() };
                        number_ids.set(cell.coords(), Some(numbers.len()));
                    }
                    (State::Idle, _) => {}
                    (State::Number { accum, ident }, Cell::Digit(n)) => {
                        *accum = 10*(*accum) + (n as u32);
                        number_ids.set(cell.coords(), Some(*ident));
                    }
                    (State::Number { accum, ident }, _) => {
                        assert_eq!(numbers.len(), *ident);
                        numbers.push(*accum);
                        state = State::Idle;
                    }
                }
            }

            if let State::Number { accum, ident: _ } = state {
                numbers.push(accum);
            }
        }

        NumberMap { numbers, number_ids }
    }
}

fn solve1(grid: &Input) -> Result<u64> {
    let map = NumberMap::from_grid(grid);
    let mut used_ids = HashSet::new();

    // build set of symbol-adjacent number IDs (i.e. those which are considered part numbers)
    for cell in grid.points().filter(|c| matches!(**c, Cell::Symbol(_))) {
        used_ids.extend(cell.neighbors()
                       .filter_map(|c| *map.number_ids.get(c.coords())));
    }

    Ok(used_ids.into_iter()
               .map(|id| map.numbers[id])
               .sum::<u32>() as u64)
}

fn solve2(grid: &Input) -> Result<u64> {
    let map = NumberMap::from_grid(grid);

    // find all gears and store their ratios
    Ok(grid.points()
           .filter(|c| matches!(**c, Cell::Symbol('*')))
           .filter_map(|c| {
               let mut vals = [0, 0];
               let mut val_ids = HashSet::new();
               let mut n = 0;

               for digit in c.neighbors().filter(|n| matches!(**n, Cell::Digit(_))) {
                   let val_id = map.number_ids.get(digit.coords())
                               .expect("Grid consistency error");
                   if val_ids.contains(&val_id) {
                       continue;
                   }

                   if n == 2 {
                       return None;
                   }

                   vals[n] = map.numbers[val_id];
                   val_ids.insert(val_id);
                   n += 1;
               }

               if n < 2 {
                   return None;
               }
               Some(vals[0] * vals[1])
           })
           .sum::<u32>() as u64)
}

problem!(crate::util::load_grid => Grid<Cell> => (solve1, solve2));
