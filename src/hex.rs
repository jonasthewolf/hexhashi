
use std::{
    cmp, collections::BTreeMap, fmt::{Debug, Display}
};

use itertools::Itertools;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::hashi::{Bridge, BridgeState, CoordinateSystem, Island};

#[derive(Debug)]
pub struct HexBridge {
    state: BridgeState,
}

///
///
/// Linear space:
/// 0 is top left
/// All even rows have one more column.
///
#[derive(Debug)]
pub struct HexSystem {
    columns: usize,
    rows: usize,
    islands: Vec<Island>,
    bridges: BTreeMap<(usize, usize), HexBridge>,
}

impl Display for HexSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut even_row = true;
        let mut last_end_index = self.columns - 1;
        dbg!(last_end_index);
        f.write_fmt(format_args!("\u{250f}{:\u{2501}<width$}\u{2513}\n", "", width = 2 * self.columns + 1))?;
        for index in 0..self.islands.len() {
            if index == last_end_index + if even_row { 1 } else { 0 } - self.columns {
                f.write_fmt(format_args!("\u{2503}"))?;
                if even_row {
                    f.write_str(" ")?;
                }
            }
            if let Some(island) = self.islands[index] {
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
        f.write_fmt(format_args!("\u{2517}{:\u{2501}<width$}\u{251b}", "", width = 2 * self.columns + 1))?;
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
        let mut bridges = BTreeMap::new();

        let mut rng = SmallRng::seed_from_u64(seed);

        let mut cur_index = rng.random_range(0..size);

        // Randomly walk a tour on the grid selection direction, width and length of bridge
        // TODO Check for collisions
        while bridges.keys().flat_map(|(a,b)| [a,b]).unique().count() < num_islands {
            dbg!(cur_index);
            let cur_connections = HexSystem::get_connected_islands(max_columns, max_rows, cur_index);
            let direction = rng.random_range(0..cur_connections.len());
            dbg!(direction);
            let mut bridge_length = rng.random_range(1..=max_bridge_length);
            dbg!(bridge_length);
            let bridge_width = rng.random_range(1..=2);
            dbg!(bridge_width);
            let mut next_index = cur_connections[direction];
            loop {
                let next_connections = HexSystem::get_connected_islands(max_columns, max_rows, next_index);
                bridge_length -= 1;
                if bridge_length == 0 || next_connections.len() == cur_connections.len() {
                    break;
                }
                next_index = cur_connections[direction];
            }
            bridges.entry((cmp::min(cur_index, next_index), cmp::max(cur_index, next_index))).and_modify(|e: &mut HexBridge| { e.state = match e.state {
                BridgeState::Empty => BridgeState::Partial,
                BridgeState::Partial => BridgeState::Full,
                BridgeState::Full => BridgeState::Full,
                BridgeState::Blocked => unreachable!(),
            };}).or_insert_with(|| {
                HexBridge {
                    state: match bridge_width {
                        1 => BridgeState::Partial,
                        2 => BridgeState::Full,
                        _ => unreachable!(),
                    },
                }
            });
            cur_index = next_index;
        }
        dbg!(&bridges);
        // Create islands from bridges
        let mut islands = vec![None; size];
        let mut island_indices = bridges
            .keys()
            .flat_map(|(a, b)| [*a, *b])
            .collect::<Vec<_>>();
        island_indices.sort();
        island_indices.dedup();
        dbg!(&island_indices);
        for i in island_indices {
            islands[i] = Some(
                bridges
                    .iter()
                    .map(|((f, t), b)| {
                        if f == &i || t == &i {
                            match b.state {
                                BridgeState::Partial => 1,
                                BridgeState::Full => 2,
                                _ => unreachable!(),
                            }
                        } else {
                            0
                        }
                    })
                    .sum(),
            )
        }

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
    fn get_connected_islands(columns: usize, rows: usize, from: usize) -> Vec<usize> {
        let mut connections = vec![];
        let mut first_column = 0;
        let mut last_column = columns - 1;
        let mut even_row = true;
        while last_column < from {
            first_column += columns + if even_row { 0 } else { 1 };
            last_column += columns + if even_row { 1 } else { 0 };
            even_row = !even_row;
        }
        // Starting from second row
        if from >= columns {
            if even_row || from != first_column {
                connections.push(from - columns - 1);
            }
            if from != last_column + if even_row { 1 } else { 0 } {
                connections.push(from - columns);
            }
        }
        // First column
        if from != first_column {
            connections.push(from - 1);
        }
        // Last column
        if from != last_column {
            connections.push(from + 1);
        }
        // Not last row
        if from <= (rows - 1) * columns + 1 {
            if even_row || from != first_column {
                connections.push(from + columns);
            }
            if from != last_column + if even_row { 1 } else { 0 } {
                connections.push(from + columns + 1);
            }
        }
        connections
    }

    ///
    /// 
    /// 
    fn get_crossing_bridge(columns: usize, rows: usize, from: usize, to: usize) -> (usize, usize) {
        // let mut first_column = 0;
        // let mut last_column = columns - 1;
        // let mut even_row = true;
        // while last_column < from {
        //     first_column += columns + if even_row { 0 } else { 1 };
        //     last_column += columns + if even_row { 1 } else { 0 };
        //     even_row = !even_row;
        // }
        // // Starting from second row
        // if from >= columns {
        //     if even_row || from != first_column {
        //         connections.push(from - columns - 1);
        //     }
        //     if from != last_column + if even_row { 1 } else { 0 } {
        //         connections.push(from - columns);
        //     }
        // }
        // // First column
        // if from != first_column {
        //     connections.push(from - 1);
        // }
        // // Last column
        // if from != last_column {
        //     connections.push(from + 1);
        // }
        // // Not last row
        // if from <= (rows - 1) * columns + 1 {
        //     if even_row || from != first_column {
        //         connections.push(from + columns);
        //     }
        //     if from != last_column + if even_row { 1 } else { 0 } {
        //         connections.push(from + columns + 1);
        //     }
        // }
        (0,0)
    }

    ///
    /// Get size of vector needed to store a `columns` x `rows` puzzle.
    ///
    fn get_size(columns: usize, rows: usize) -> usize {
        columns * rows + rows / 2
    }
}

impl CoordinateSystem for HexSystem {
    ///
    /// Get connected islands for `from` island.
    /// TODO Kanten dürfen nicht mehr auf "Leere Inseln" zeigen.
    ///
    fn get_connected_islands(&self, from: usize) -> Vec<usize> {
        self.bridges
            .iter()
            .filter_map(|((island, other), _)| if island == &from { Some(*other) } else { None })
            .collect()
    }

    fn get_bridges(&self, from: usize) -> Vec<&BridgeState> {
        self.bridges
            .iter()
            .filter_map(|((island, _), bridge)| {
                if island == &from {
                    Some(bridge.get_state())
                } else {
                    None
                }
            })
            .collect()
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

    #[test]
    fn check_connections() {
        for i in 0..22 {
            let x = HexSystem::get_connected_islands(4, 5, i);
            let res: Option<Vec<usize>> = match i {
                0 => Some(vec![1, 4, 5]),
                1 => Some(vec![0, 2, 5, 6]),
                2 => Some(vec![1, 3, 6, 7]),
                3 => Some(vec![2, 7, 8]),
                4 => Some(vec![0, 5, 9]),
                5 => Some(vec![0, 1, 4, 6, 9, 10]),
                6 => Some(vec![1, 2, 5, 7, 10, 11]),
                7 => Some(vec![2, 3, 6, 8, 11, 12]),
                8 => Some(vec![3, 7, 12]),
                9 => Some(vec![4, 5, 10, 13, 14]),
                10 => Some(vec![5, 6, 9, 11, 14, 15]),
                11 => Some(vec![6, 7, 10, 12, 15, 16]),
                12 => Some(vec![7, 8, 11, 16, 17]),
                13 => Some(vec![9, 14, 18]),
                14 => Some(vec![9, 10, 13, 15, 18, 19]),
                15 => Some(vec![10, 11, 14, 16, 19, 20]),
                16 => Some(vec![11, 12, 15, 17, 20, 21]),
                17 => Some(vec![12, 16, 21]),
                18 => Some(vec![13, 14, 19]),
                19 => Some(vec![14, 15, 18, 20]),
                20 => Some(vec![15, 16, 19, 21]),
                21 => Some(vec![16, 17, 20]),
                _ => None,
            };
            if let Some(res) = res {
                assert_eq!(res, x);
            } else {
                panic!("must not happen.")
            }
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
}
