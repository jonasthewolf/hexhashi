pub type Island = Option<usize>;

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum BridgeState {
    Empty,
    Partial,
    Full,
    Blocked,
}

pub trait CoordinateSystem {
    fn get_connected_islands(&self, from: usize) -> Vec<usize>;

    fn get_bridges(&self, from: usize) -> Vec<&BridgeState>;
}

pub trait Bridge {
    fn cycle(&mut self) -> Option<usize>;

    fn get_count(&self) -> usize;

    fn get_max(&self) -> usize;

    fn get_state(&self) -> &BridgeState;
}
