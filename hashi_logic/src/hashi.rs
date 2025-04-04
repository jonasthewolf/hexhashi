pub type Island = Option<ActualIsland>;

#[derive(Clone, Debug)]
pub struct ActualIsland {
    pub target_bridges: usize,
    pub current_bridges: usize,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq)]
pub enum BridgeState {
    Empty,
    Partial,
    Full,
    Blocked,
}

pub trait CoordinateSystem {
    fn get_connected_islands(&self, from: usize) -> Vec<usize>;

    fn get_mut_bridge(&mut self, from: usize, to: usize) -> Option<&mut impl Bridge>;

    fn get_row_column_for_index(&self, from: usize) -> (usize, usize);
}

pub trait Bridge {
    fn cycle(&mut self) -> Option<usize>;

    fn get_count(&self) -> usize;

    fn get_max(&self) -> usize;

    fn get_state(&self) -> &BridgeState;
}
