use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use rand::prelude::*;

use crate::hashi::{Bridge, BridgeState, CoordinateSystem, Island};

#[derive(Clone, Debug)]
pub struct HexBridge {
    target: BridgeState,
    state: BridgeState,
}

///
///
/// Linear space:
/// 0 is top left
/// All even rows have one more column.
///
#[derive(Clone, Debug)]
pub struct HexSystem {
    pub columns: usize,
    pub rows: usize,
    pub islands: Vec<Island>,
    pub bridges: BTreeMap<(usize, usize), HexBridge>,
}

impl Display for HexSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut even_row = true;
        let mut last_end_index = self.columns - 1;
        f.write_fmt(format_args!(
            "\u{250f}{:\u{2501}<width$}\u{2513}\n",
            "",
            width = 2 * self.columns + 1
        ))?;
        for index in 0..self.islands.len() {
            if index == last_end_index + if even_row { 1 } else { 0 } - self.columns {
                f.write_fmt(format_args!("\u{2503}"))?;
                if even_row {
                    f.write_str(" ")?;
                }
            }
            if let Some(island) = &self.islands[index] {
                f.write_fmt(format_args!("{}", island))?;
            } else {
                f.write_str(" ")?;
            }
            if even_row || index != last_end_index {
                f.write_str(" ")?;
            }
            if index == last_end_index {
                f.write_str("\u{2503}\n")?;
                even_row = !even_row;
                last_end_index = last_end_index + self.columns + if even_row { 0 } else { 1 };
            }
        }
        f.write_fmt(format_args!(
            "\u{2517}{:\u{2501}<width$}\u{251b}",
            "",
            width = 2 * self.columns + 1
        ))?;
        Ok(())
    }
}

impl HexSystem {
    pub fn generate_new(
        seed: u64,
        max_columns: usize,
        max_rows: usize,
        num_islands: usize,
        max_bridge_length: usize,
        _ratio_big_island: f64,
        _ratio_long_bridge: f64,
    ) -> Self {
        let size = HexSystem::get_size(max_columns, max_rows);

        let rng = SmallRng::seed_from_u64(seed);

        let islands = HexSystem::generate_islands(
            max_columns,
            max_rows,
            num_islands,
            max_bridge_length,
            size,
            rng,
        );

        // TODO crop to minimum necessary size

        HexSystem {
            columns: max_columns,
            rows: max_rows,
            islands,
            bridges: BTreeMap::new(),
        }
    }

    ///
    /// Get indices of connected islands
    ///
    /// Skip first row here, if `from` is in first row of puzzle (i.e. x < c).
    /// Skip last row here, if `from` is in last row of puzzle (i.e. x > w * h - c).
    /// Skip first column here, if `from` is in first column of puzzle.
    /// Skip last column here, if `from` is in last column of puzzle.
    ///
    ///  x - c - 1 ------ x - c
    ///     /      \    /     \
    ///  x - 1 ----- x ----- x + 1
    ///     \      /   \      /
    ///    x + c  ------ x + c + 1
    ///
    /// The order is NW, NE, E, SE, SW, W
    ///
    ///
    const fn get_connected_islands(columns: usize, rows: usize, from: usize) -> [Option<usize>; 6] {
        let mut connections = [None; 6];
        let even_row = from % (2 * columns + 1) < columns;
        let first_column = from - from % (2 * columns + 1) + if even_row { 0 } else { columns };
        let last_column = first_column + columns - 1 + if even_row { 0 } else { 1 };
        // Starting from second row
        if from >= columns {
            if even_row || from != first_column {
                connections[0] = Some(from - columns - 1);
            }
            if from != last_column + if even_row { 1 } else { 0 } {
                connections[1] = Some(from - columns);
            }
        }
        // First column
        if from != first_column {
            connections[5] = Some(from - 1);
        }
        // Last column
        if from != last_column {
            connections[2] = Some(from + 1);
        }
        // Not last row
        if from <= (rows - 1) * columns + 1 {
            if even_row || from != first_column {
                connections[4] = Some(from + columns);
            }
            if from != last_column + if even_row { 1 } else { 0 } {
                connections[3] = Some(from + columns + 1);
            }
        }
        connections
    }

    ///
    /// Get size of vector needed to store a `columns` x `rows` puzzle.
    ///
    fn get_size(columns: usize, rows: usize) -> usize {
        columns * rows + rows / 2
    }

    ///
    ///
    ///
    ///
    fn generate_islands(
        max_columns: usize,
        max_rows: usize,
        num_islands: usize,
        max_bridge_length: usize,
        size: usize,
        mut rng: SmallRng,
    ) -> Vec<Island> {
        #[derive(Clone, Debug, PartialEq, Eq)]
        enum GenIsland {
            Blank,
            Created,
        }

        let mut indices = vec![None; HexSystem::get_size(max_columns, max_rows)];
        let mut start_index = rng.random_range(0..size);
        indices[start_index] = Some(GenIsland::Created);
        let mut bridges: BTreeMap<(usize, usize), HexBridge> = BTreeMap::new();

        let mut limit = 50;

        // Randomly walk a tour on the grid randomly selecting direction, width and length of bridge
        while indices
            .iter()
            .filter(|i| matches!(i, Some(GenIsland::Created)))
            .count()
            < num_islands && limit > 0
        {
            let direction = rng.random_range(0..6);
            let mut bridge_length = rng.random_range(1..=max_bridge_length);
            let orig_bridge_length = bridge_length;
            let bridge_width = rng.random_range(1..=2);

            // Keep direction until any of the following applies:
            // a) direction is not available anymore (basically edge is hit), or
            // b) `bridge_length` is reached, or
            // c) an existing island is reached, or
            // d) the bridge is blocked (i.e. the index is marked as blocked).
            dbg!(start_index);
            {
                let even_row = start_index % (2 * max_columns + 1) < max_columns;
                let row = 2 * (start_index / (2 * max_columns + 1)) + if even_row { 0 } else { 1 };
                let column =
                    start_index % (2 * max_columns + 1) - if even_row { 0 } else { max_columns };
                dbg!(row, column);
            }
            dbg!(direction);
            let mut next_index = start_index;
            // Loop terminates at latest, when bridge length is reached.
            let end_index = loop {
                let next_connections =
                    HexSystem::get_connected_islands(max_columns, max_rows, next_index);
                // a)
                if let Some(i) = next_connections[direction] {
                    next_index = i;
                } else {
                    dbg!(next_index);
                    break next_index;
                };
                bridge_length -= 1;
                dbg!(bridge_length);
                // b), c) and d)
                if bridge_length == 0 || indices[next_index] != None {
                    dbg!(next_index);
                    break next_index;
                }
                // Mark island as blank.
                if orig_bridge_length > 1 {
                    dbg!(orig_bridge_length);
                    indices[next_index] = Some(GenIsland::Blank);
                }
                dbg!(next_index);
            };
            if start_index != end_index && indices[end_index] != Some(GenIsland::Blank) {
                dbg!(start_index);
                dbg!(end_index);
                bridges
                    .entry((
                        std::cmp::min(start_index, end_index),
                        std::cmp::max(start_index, end_index),
                    ))
                    .and_modify(|e| {
                        (*e).target = match e.target {
                            BridgeState::Empty => unreachable!(),
                            BridgeState::Partial => BridgeState::Full,
                            BridgeState::Full => BridgeState::Full,
                            BridgeState::Blocked => unreachable!(),
                        };
                    })
                    .or_insert(HexBridge {
                        target: match bridge_width {
                            1 => BridgeState::Partial,
                            2 => BridgeState::Full,
                            _ => unreachable!(),
                        },
                        state: BridgeState::Empty,
                    });
                indices[end_index] = Some(GenIsland::Created);
                start_index = end_index;
            } else {
                // let x = indices.iter().enumerate().filter_map(|(a,b)| if b.is_some() {Some(format!("{}:{:?}", a, b.clone().unwrap()))} else {None}).collect::<Vec<_>>();
                // dbg!(x);
                limit -= 1;
            }
        }
        // Create islands from bridges
        let mut islands = vec![None; indices.len()];
        bridges.iter_mut().for_each(|((i1, i2), bw)| {
            let mut apply = |i: usize| {
                let is = islands[i].get_or_insert(0);
                *is += match bw.target {
                    BridgeState::Empty => 0,
                    BridgeState::Partial => 1,
                    BridgeState::Full => 2,
                    BridgeState::Blocked => 0,
                };
            };
            apply(*i1);
            apply(*i2);
        });
        islands
    }

    ///
    ///
    ///
    fn crop(islands: &mut Vec<Island>, max_columns: usize) -> (usize, usize) {
        (0, 0)
    }
}
impl CoordinateSystem for HexSystem {
    ///
    /// Get connected islands for `from` island.
    /// TODO Vorbedingung: Kanten dürfen nicht mehr auf "Leere Inseln" zeigen.
    ///
    fn get_connected_islands(&self, from: usize) -> Vec<usize> {
        self.bridges
            .iter()
            .filter_map(|((island, other), _)| if island == &from { Some(*other) } else { None })
            .collect()
    }

    ///
    ///
    ///
    fn get_mut_bridge(&mut self, from: usize, to: usize) -> Option<&mut impl Bridge> {
        self.bridges
            .get_mut(&(std::cmp::min(from, to), std::cmp::max(from, to)))
    }

    ///
    /// Get row, column for `from` index of island.
    ///
    fn get_row_column_for_index(&self, from: usize) -> (usize, usize) {
        let even_row = from % (2 * self.columns + 1) < self.columns;
        let row = 2 * (from / (2 * self.columns + 1)) + if even_row { 0 } else { 1 };
        let column = from % (2 * self.columns + 1) - if even_row { 0 } else { self.columns };
        (row, column)
    }

    fn get_actual_bridges(&self, from: usize) -> usize {
        let connections = HexSystem::get_connected_islands(self.columns, self.rows, from);
        // connections.iter().map(|c| self.bridges)
        // self.bridges
        // .get(&(std::cmp::min(from, to), std::cmp::max(from, to))).
        0
    }
}

impl Bridge for HexBridge {
    fn cycle(&mut self) -> Option<usize> {
        match self.target {
            BridgeState::Empty => self.target = BridgeState::Partial,
            BridgeState::Partial => self.target = BridgeState::Full,
            BridgeState::Full => self.target = BridgeState::Empty,
            BridgeState::Blocked => self.target = BridgeState::Blocked,
        }
        match self.target {
            BridgeState::Empty => Some(0),
            BridgeState::Partial => Some(1),
            BridgeState::Full => Some(2),
            BridgeState::Blocked => None,
        }
    }

    fn get_count(&self) -> usize {
        match self.target {
            BridgeState::Empty => 0,
            BridgeState::Partial => 1,
            BridgeState::Full => 2,
            BridgeState::Blocked => 0,
        }
    }

    fn get_max(&self) -> usize {
        2
    }

    fn get_state(&self) -> &BridgeState {
        &self.target
    }
}

#[cfg(test)]
mod test {
    use super::HexSystem;

    // NW, NE, E, SE, SW, W
    #[test]
    fn check_connections() {
        for i in 0..22 {
            let x = HexSystem::get_connected_islands(4, 5, i);
            let res: [Option<usize>; 6] = match i {
                0 => [None, None, Some(1), Some(5), Some(4), None],
                1 => [None, None, Some(2), Some(6), Some(5), Some(0)],
                2 => [None, None, Some(3), Some(7), Some(6), Some(1)],
                3 => [None, None, None, Some(8), Some(7), Some(2)],
                4 => [None, Some(0), Some(5), Some(9), None, None],
                5 => [Some(0), Some(1), Some(6), Some(10), Some(9), Some(4)],
                6 => [Some(1), Some(2), Some(7), Some(11), Some(10), Some(5)],
                7 => [Some(2), Some(3), Some(8), Some(12), Some(11), Some(6)],
                8 => [Some(3), None, None, None, Some(12), Some(7)],
                9 => [Some(4), Some(5), Some(10), Some(14), Some(13), None],
                10 => [Some(5), Some(6), Some(11), Some(15), Some(14), Some(9)],
                11 => [Some(6), Some(7), Some(12), Some(16), Some(15), Some(10)],
                12 => [Some(7), Some(8), None, Some(17), Some(16), Some(11)],
                13 => [None, Some(9), Some(14), Some(18), None, None],
                14 => [Some(9), Some(10), Some(15), Some(19), Some(18), Some(13)],
                15 => [Some(10), Some(11), Some(16), Some(20), Some(19), Some(14)],
                16 => [Some(11), Some(12), Some(17), Some(21), Some(20), Some(15)],
                17 => [Some(12), None, None, None, Some(21), Some(16)],
                18 => [Some(13), Some(14), Some(19), None, None, None],
                19 => [Some(14), Some(15), Some(20), None, None, Some(18)],
                20 => [Some(15), Some(16), Some(21), None, None, Some(19)],
                21 => [Some(16), Some(17), None, None, None, Some(20)],
                _ => unreachable!(),
            };
            assert_eq!(res, x);
        }
    }

    #[test]
    fn size_calc() {
        assert_eq!(HexSystem::get_size(4, 5), 22);
        assert_eq!(HexSystem::get_size(3, 3), 10);
        assert_eq!(HexSystem::get_size(6, 2), 13);
        assert_eq!(HexSystem::get_size(6, 3), 19);
        assert_eq!(HexSystem::get_size(1, 1), 1);
        assert_eq!(HexSystem::get_size(15, 14), 217);
        assert_eq!(HexSystem::get_size(15, 15), 232);
        assert_eq!(HexSystem::get_size(15, 16), 248);
        assert_eq!(HexSystem::get_size(14, 14), 203);
        assert_eq!(HexSystem::get_size(14, 15), 217);
        assert_eq!(HexSystem::get_size(14, 16), 232);
        assert_eq!(HexSystem::get_size(16, 14), 231);
        assert_eq!(HexSystem::get_size(16, 15), 247);
        assert_eq!(HexSystem::get_size(16, 16), 264);
    }

    #[test]
    fn very_small_hashi() {
        let hex = HexSystem::generate_new(1, 4, 5, 5, 2, 0.0, 0.0);
        // dbg!(&hex);
        println!("{}", hex);
    }

    #[test]
    fn small_hashi() {
        let hex = HexSystem::generate_new(1, 4, 5, 8, 3, 0.0, 0.0);
        // dbg!(&hex);
        println!("{}", hex);
    }

    #[test]
    fn medium_hashi() {
        let hex = HexSystem::generate_new(1, 15, 15, 28, 7, 0.0, 0.0);
        // dbg!(&hex);
        println!("{}", hex);
    }
}
