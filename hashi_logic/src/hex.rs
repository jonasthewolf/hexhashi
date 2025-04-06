use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Debug, Display},
};

use rand::prelude::*;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq)]
pub enum BridgeState {
    Empty,
    Partial,
    Full,
}
#[derive(Clone, Debug)]
pub struct HexBridge {
    state: BridgeState,
}

///
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Island {
    Empty,
    Bridged(usize), // Target number of bridges
    Blocked,
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
            if let Island::Bridged(bridges) = &self.islands[index] {
                f.write_fmt(format_args!("{}", bridges))?;
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
        let (islands, bridges, (columns, rows)) = HexSystem::generate_islands(
            max_columns,
            max_rows,
            num_islands,
            max_bridge_length,
            seed,
        );

        HexSystem {
            columns,
            rows,
            islands,
            bridges,
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
    pub const fn get_connected_indices(
        columns: usize,
        rows: usize,
        from: usize,
    ) -> [Option<usize>; 6] {
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
        seed: u64,
    ) -> (
        Vec<Island>,
        BTreeMap<(usize, usize), HexBridge>,
        (usize, usize),
    ) {
        let size = HexSystem::get_size(max_columns, max_rows);

        let mut rng = SmallRng::seed_from_u64(seed);

        let mut indices = vec![Island::Empty; HexSystem::get_size(max_columns, max_rows)];
        let mut start_index = rng.random_range(0..size);
        indices[start_index] = Island::Bridged(0);
        let mut bridges: BTreeMap<(usize, usize), HexBridge> = BTreeMap::new();

        let mut limit = 50;

        // Randomly walk a tour on the grid randomly selecting direction, width and length of bridge
        while indices
            .iter()
            .filter(|i| matches!(i, Island::Bridged(_)))
            .count()
            < num_islands
            && limit > 0
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
            let mut next_index = start_index;
            // Loop terminates at latest, when bridge length is reached.
            let end_index = loop {
                let next_connections =
                    HexSystem::get_connected_indices(max_columns, max_rows, next_index);
                // a)
                if let Some(i) = next_connections[direction] {
                    next_index = i;
                } else {
                    // dbg!(next_index);
                    break next_index;
                };
                bridge_length -= 1;
                // dbg!(bridge_length);
                // b), c) and d)
                if bridge_length == 0 || indices[next_index] != Island::Empty {
                    // dbg!(next_index);
                    break next_index;
                }
                // Mark island as blank.
                if orig_bridge_length > 1 {
                    // dbg!(orig_bridge_length);
                    indices[next_index] = Island::Blocked;
                }
                // dbg!(next_index);
            };
            if start_index != end_index && indices[end_index] != Island::Blocked {
                // dbg!(start_index);
                // dbg!(end_index);
                bridges
                    .entry((
                        std::cmp::min(start_index, end_index),
                        std::cmp::max(start_index, end_index),
                    ))
                    .and_modify(|e| {
                        e.state = match e.state {
                            BridgeState::Empty => unreachable!(),
                            BridgeState::Partial => BridgeState::Full,
                            BridgeState::Full => BridgeState::Full,
                        };
                    })
                    .or_insert(HexBridge {
                        state: match bridge_width {
                            1 => BridgeState::Partial,
                            2 => BridgeState::Full,
                            _ => unreachable!(),
                        },
                    });
                indices[end_index] = Island::Bridged(0);
                start_index = end_index;
            } else {
                // let x = indices.iter().enumerate().filter_map(|(a,b)| if b.is_some() {Some(format!("{}:{:?}", a, b.clone().unwrap()))} else {None}).collect::<Vec<_>>();
                // // dbg!(x);
                limit -= 1;
            }
        }
        // Create islands from bridges
        let mut islands: Vec<Island> = vec![Island::Empty; indices.len()];
        bridges.iter_mut().for_each(|((i1, i2), bw)| {
            let mut apply = |i: usize| {
                let is = &mut islands[i];
                let width = match bw.state {
                    BridgeState::Empty => 0,
                    BridgeState::Partial => 1,
                    BridgeState::Full => 2,
                };
                *is = match is {
                    Island::Empty => Island::Bridged(width),
                    Island::Bridged(c) => Island::Bridged(*c + width),
                    Island::Blocked => Island::Empty,
                }
            };
            apply(*i1);
            apply(*i2);
            // Reset bridge state, otherwise puzzle would be returned solved.
            bw.state = BridgeState::Empty;
        });
        // Fill bridges between existing islands that do not contribute to solution.
        HexSystem::fill_bridges(&islands, max_columns, max_rows, &mut bridges);
        let new_size = HexSystem::crop(&mut islands, max_columns, max_rows);
        (islands, bridges, new_size)
    }

    ///
    /// Returns the new size (columns, rows)
    ///
    fn crop(islands: &mut Vec<Island>, max_columns: usize, max_rows: usize) -> (usize, usize) {
        // TODO Implement
        (max_columns, max_rows)
    }

    ///
    ///
    ///
    fn fill_bridges(
        islands: &[Island],
        columns: usize,
        rows: usize,
        bridges: &mut BTreeMap<(usize, usize), HexBridge>,
    ) {
        for start_index in 0..islands.len() {
            if let Island::Bridged(_) = islands[start_index] {
                let connections = HexSystem::get_connected_indices(columns, rows, start_index);
                for (direction, opt_con) in connections.iter().enumerate() {
                    let mut end_index = None;
                    if let Some(con) = *opt_con {
                        match islands[con] {
                            Island::Blocked => unreachable!(),
                            Island::Bridged(_) => {
                                end_index = Some(con);
                            }
                            Island::Empty => {
                                let mut next_index = con;
                                loop {
                                    let next_con =
                                        HexSystem::get_connected_indices(columns, rows, next_index)
                                            [direction];
                                    if let Some(next_island) = next_con {
                                        if let Island::Bridged(_) = islands[next_island] {
                                            end_index = Some(next_island);
                                            break;
                                        }
                                        if let Island::Empty = islands[next_island] {
                                            next_index = next_island;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                        if let Some(end_index) = end_index {
                            bridges.insert(
                                (
                                    std::cmp::min(start_index, end_index),
                                    std::cmp::max(start_index, end_index),
                                ),
                                HexBridge {
                                    state: BridgeState::Empty,
                                },
                            );
                            break;
                        }
                    }
                }
            }
        }
    }

    ///
    /// Get connected islands for `from` island.
    ///
    fn get_connected_islands(&self, from: usize) -> Vec<usize> {
        self.bridges
            .iter()
            .filter_map(|((island, other), _)| {
                if island == &from {
                    Some(*other)
                } else if other == &from {
                    Some(*island)
                } else {
                    None
                }
            })
            .collect()
    }

    ///
    ///
    ///
    pub fn get_mut_bridge(&mut self, from: usize, to: usize) -> Option<&mut HexBridge> {
        self.bridges
            .get_mut(&(std::cmp::min(from, to), std::cmp::max(from, to)))
    }

    ///
    ///
    ///
    pub fn get_bridge(&self, from: usize, to: usize) -> Option<&HexBridge> {
        self.bridges
            .get(&(std::cmp::min(from, to), std::cmp::max(from, to)))
    }

    ///
    /// Get row, column for `from` index of island.
    ///
    pub fn get_row_column_for_index(&self, from: usize) -> (usize, usize) {
        let even_row = from % (2 * self.columns + 1) < self.columns;
        let row = 2 * (from / (2 * self.columns + 1)) + if even_row { 0 } else { 1 };
        let column = from % (2 * self.columns + 1) - if even_row { 0 } else { self.columns };
        (row, column)
    }

    ///
    ///
    ///
    ///
    pub fn get_actual_bridges(&self, from: usize) -> usize {
        let connections = self.get_connected_islands(from);
        connections
            .into_iter()
            .filter_map(|to| {
                self.bridges
                    .get(&(std::cmp::min(from, to), std::cmp::max(from, to)))
                    .map(|b| b.get_count())
            })
            .sum()
    }

    ///
    ///
    ///
    ///
    pub fn is_solved(&self) -> bool {
        let mut bridged_islands = self
            .islands
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if let Island::Bridged(_) = t {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        let mut visited_islands = BTreeSet::new();
        let start_island = self
            .islands
            .iter()
            .enumerate()
            .filter_map(|(index, target)| {
                if let Island::Bridged(target) = target {
                    Some((index, *target))
                } else {
                    None
                }
            })
            .nth(0)
            .unwrap();
        visited_islands.insert(start_island.0);
        bridged_islands.remove(&start_island.0);
        let mut next_islands: Vec<usize> = self
            .get_connected_islands(start_island.0)
            .into_iter()
            .filter(|to| {
                self.bridges
                    .get(&(
                        std::cmp::min(start_island.0, *to),
                        std::cmp::max(start_island.0, *to),
                    ))
                    .map(|b| b.get_count())
                    .is_some_and(|x| x > 0)
            })
            .collect::<Vec<_>>();
        loop {
            for ni in &next_islands {
                if !visited_islands.contains(ni) {
                    if self.islands[*ni] == Island::Bridged(self.get_actual_bridges(*ni)) {
                        bridged_islands.remove(ni);
                    } else {
                        return false;
                    }
                    visited_islands.insert(*ni);
                }
            }
            next_islands = next_islands
                .iter()
                .flat_map(|i| {
                    self.get_connected_islands(*i).into_iter().filter(|to| {
                        self.bridges
                            .get(&(std::cmp::min(*i, *to), std::cmp::max(*i, *to)))
                            .map(|b| b.get_count())
                            .is_some_and(|x| x > 0)
                            && !visited_islands.contains(to)
                    })
                })
                .collect::<Vec<_>>();
            if next_islands.is_empty() {
                break;
            }
        }
        bridged_islands.is_empty()
    }
}

impl HexBridge {
    pub fn cycle(&mut self) -> Option<usize> {
        self.state = match self.state {
            BridgeState::Empty => BridgeState::Partial,
            BridgeState::Partial => BridgeState::Full,
            BridgeState::Full => BridgeState::Empty,
        };
        match self.state {
            BridgeState::Empty => Some(0),
            BridgeState::Partial => Some(1),
            BridgeState::Full => Some(2),
        }
    }

    pub fn get_count(&self) -> usize {
        match self.state {
            BridgeState::Empty => 0,
            BridgeState::Partial => 1,
            BridgeState::Full => 2,
        }
    }

    pub fn get_state(&self) -> &BridgeState {
        &self.state
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::{BridgeState, Island};

    use super::{HexBridge, HexSystem};

    // NW, NE, E, SE, SW, W
    #[test]
    fn check_connections() {
        for i in 0..22 {
            let x = HexSystem::get_connected_indices(4, 5, i);
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
        // // dbg!(&hex);
        println!("{}", hex);
    }

    #[test]
    fn small_hashi() {
        let hex = HexSystem::generate_new(1, 4, 5, 8, 3, 0.0, 0.0);
        // // dbg!(&hex);
        println!("{}", hex);
    }

    #[test]
    fn medium_hashi() {
        let hex = HexSystem::generate_new(1, 15, 15, 28, 7, 0.0, 0.0);
        // // dbg!(&hex);
        println!("{}", hex);
    }

    #[test]
    fn random_hashi() {
        let hex = HexSystem::generate_new(63, 10, 10, 40, 10, 0.0, 0.0);
        // dbg!(&hex);
        println!("{}", hex);
    }

    #[test]
    fn solution_check() {
        let mut islands = vec![Island::Empty; 22];
        islands[0] = Island::Bridged(2);
        islands[1] = Island::Bridged(2);
        let bridges = BTreeMap::from([(
            (0usize, 1usize),
            HexBridge {
                state: BridgeState::Full,
            },
        )]);
        let hex = HexSystem {
            columns: 4,
            rows: 5,
            islands,
            bridges,
        };
        assert!(hex.is_solved());
    }

    #[test]
    fn solution_check_complex() {
        let mut islands = vec![Island::Empty; 22];
        islands[0] = Island::Bridged(2);
        islands[1] = Island::Bridged(3);
        islands[4] = Island::Bridged(1);
        islands[5] = Island::Bridged(2);
        let bridges = BTreeMap::from([
            (
                (0usize, 1usize),
                HexBridge {
                    state: BridgeState::Full,
                },
            ),
            (
                (0usize, 4usize),
                HexBridge {
                    state: BridgeState::Empty,
                },
            ),
            (
                (0usize, 5usize),
                HexBridge {
                    state: BridgeState::Empty,
                },
            ),
            (
                (1usize, 5usize),
                HexBridge {
                    state: BridgeState::Partial,
                },
            ),
            (
                (4usize, 5usize),
                HexBridge {
                    state: BridgeState::Partial,
                },
            ),
        ]);
        let hex = HexSystem {
            columns: 4,
            rows: 5,
            islands,
            bridges,
        };
        assert!(hex.is_solved());
    }

    #[test]
    fn fill_bridges_small() {
        let mut islands = vec![Island::Empty; 22];
        islands[0] = Island::Bridged(1);
        islands[2] = Island::Bridged(1);
        islands[3] = Island::Bridged(1);
        islands[15] = Island::Bridged(1);
        let mut bridges = BTreeMap::new();
        HexSystem::fill_bridges(&islands, 4, 5, &mut bridges);
        assert_eq!(
            bridges.keys().collect::<Vec<_>>(),
            vec![&(0usize, 2usize), &(0, 15), &(2, 3), &(3, 15)]
        );
        assert!(bridges.values().all(|b| b.state == BridgeState::Empty));
    }

    #[test]
    fn solution_unsolvable() {
        let mut islands = vec![Island::Empty; 22];
        islands[0] = Island::Bridged(2);
        islands[1] = Island::Bridged(3);
        let bridges = BTreeMap::from([(
            (0usize, 1usize),
            HexBridge {
                state: BridgeState::Full,
            },
        )]);
        let hex = HexSystem {
            columns: 4,
            rows: 5,
            islands,
            bridges,
        };
        assert!(!hex.is_solved());
    }

    #[test]
    fn solution_not_solved() {
        let mut islands = vec![Island::Empty; 22];
        islands[0] = Island::Bridged(2);
        islands[1] = Island::Bridged(2);
        let bridges = BTreeMap::from([(
            (0usize, 1usize),
            HexBridge {
                state: BridgeState::Partial,
            },
        )]);
        let hex = HexSystem {
            columns: 4,
            rows: 5,
            islands,
            bridges,
        };
        assert!(!hex.is_solved());
    }
}
