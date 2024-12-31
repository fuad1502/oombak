use bitvec::vec::BitVec;

#[derive(Clone)]
pub struct Wave {
    pub signal_name: String,
    pub width: usize,
    pub values: Vec<BitVec>,
}
