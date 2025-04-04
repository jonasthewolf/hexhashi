use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use rand::prelude::*;

use crate::hashi::{ActualIsland, Bridge, BridgeState, CoordinateSystem, Island};

#[derive(Clone, Debug)]
pub struct HexBridge {
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
                f.write_fmt(format_args!("{}", island.target_bridges))?;
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
        let mut islands: Vec<Island> = vec![None; size];
        let mut blank_indices = vec![];
        let mut cur_index = rng.random_range(0..size);

        // Randomly walk a tour on the grid randomly selecting direction, width and length of bridge
        while islands.iter().filter(|i| i.is_some()).count() < num_islands {
            let mut next_connections =
                HexSystem::get_connected_islands(max_columns, max_rows, cur_index);
            let direction = next_connections
                .iter()
                .enumerate()
                .filter_map(|(i, x)| if x.is_some() { Some(i) } else { None })
                .choose(&mut rng)
                .unwrap(); // unwrap ok, since, there is always a theoretical connection to the next island.
            let mut bridge_length = rng.random_range(1..=max_bridge_length);
            let orig_bridge_length = bridge_length;
            let bridge_width = rng.random_range(1..=2);
            let mut final_index = next_connections[direction].unwrap(); // unwrap ok, since only valid direction is chosen.

            // Keep direction until any of the following applies:
            // a) direction is not available anymore (basically edge is hit), or
            // b) `bridge_length` is reached, or
            // c) an island is reached.
            // d) the bridge is blocked (i.e. the edge is marked as blocked), or

            loop {
                // a)
                let Some(next_index) = next_connections[direction] else {
                    break;
                };
                final_index = next_index;
                bridge_length -= 1;
                // b)
                if bridge_length == 0 {
                    break;
                }
                let already_island = islands[next_index].is_some();
                // c)
                if already_island {
                    break;
                }
                // d)
                if blank_indices.contains(&next_index) {
                    break;
                }
                // Mark island as blank.
                if orig_bridge_length > 1 {
                    blank_indices.push(next_index);
                }

                next_connections =
                    HexSystem::get_connected_islands(max_columns, max_rows, next_index);
            }
            // It can happen that an index is marked as blank, but ends up as the final index with an island created.
            // Remove it here again.
            if let Some(bi) = blank_indices.pop() {
                if bi != final_index {
                    blank_indices.push(bi); // Push it back again.
                }
            }
            // Create island or increase its count of bridges
            if let Some(i) = &mut islands[final_index] {
                (*i).target_bridges += 1;
            } else {
                islands[final_index] = Some(ActualIsland {
                    target_bridges: bridge_width,
                    current_bridges: 0,
                });
            }
            cur_index = final_index;
        }
        // FIXME assertion does not hold
        // dbg!(&islands.iter().enumerate().filter_map(|(i, o)| if o.is_some() { Some(i) } else { None } ).collect::<Vec<_>>());
        // dbg!(&blank_indices);
        // assert!(
        //     dbg!(islands
        //         .iter()
        //         .enumerate()
        //         .filter_map(|(i, o)| if o.is_some() { Some(i) } else { None } )
        //         .filter(|i| blank_indices.contains(i))
        //         .count())
        //         == 0
        // );
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
}

impl Bridge for HexBridge {
    fn cycle(&mut self) -> Option<usize> {
        match self.state {
            BridgeState::Empty => self.state = BridgeState::Partial,
            BridgeState::Partial => self.state = BridgeState::Full,
            BridgeState::Full => self.state = BridgeState::Empty,
            BridgeState::Blocked => self.state = BridgeState::Blocked,
        }
        match self.state {
            BridgeState::Empty => Some(0),
            BridgeState::Partial => Some(1),
            BridgeState::Full => Some(2),
            BridgeState::Blocked => None,
        }
    }

    fn get_count(&self) -> usize {
        match self.state {
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
        &self.state
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
