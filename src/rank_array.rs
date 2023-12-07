use sucds::int_vectors::{Access, PrefixSummedEliasFano};

#[derive(Default)]
pub struct EliasFanoRankArray {
    count_ranks: PrefixSummedEliasFano,
}

impl EliasFanoRankArray {
    pub fn build(count_ranks: Vec<usize>) -> Self {
        Self {
            count_ranks: PrefixSummedEliasFano::from_slice(&count_ranks).unwrap(),
        }
    }

    pub fn get(&self, i: usize) -> usize {
        self.count_ranks.access(i).unwrap()
    }
}
