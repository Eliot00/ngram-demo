use std::collections::HashMap;

use sucds::{int_vectors::CompactVector, utils::needed_bits};

use crate::{
    rank_array::EliasFanoRankArray, trie_array::EliasFanoTrieArray, vocabulary::Vocabulary,
};

#[derive(Default)]
pub struct TrieCountLmBuilder {
    loaders: Vec<Vec<(String, usize)>>,
    vocab: Vocabulary,
    arrays: Vec<EliasFanoTrieArray>,
    count_ranks: Vec<EliasFanoRankArray>,
    counts_builder: CountsBuilder,
}

impl TrieCountLmBuilder {
    pub fn new(loaders: Vec<Vec<(String, usize)>>) -> Self {
        Self {
            loaders,
            ..Default::default()
        }
    }

    fn build_counts(&mut self) {
        for loader in &self.loaders {
            for record in loader {
                self.counts_builder.eat_value(record.1);
            }
            self.counts_builder.build_sequence();
        }
    }

    fn build_vocabulary(&mut self) {
        let records = self.loaders[0].clone();
        self.vocab = Vocabulary::new(
            &records
                .iter()
                .map(|(s, _)| s.as_str())
                .collect::<Vec<&str>>(),
        );
    }
}

#[derive(Default)]
pub struct CountsBuilder {
    // Mapping from eaten values to their frequencies
    v2f_map: HashMap<usize, usize>,
    // Mappings from eaten values to their ranks
    v2r_maps: Vec<HashMap<usize, usize>>,
    // In which values are sorted in decreasing order of their frequencies
    sorted_sequences: Vec<CompactVector>,
}

impl CountsBuilder {
    pub fn release(self) -> Vec<CompactVector> {
        self.sorted_sequences
    }

    pub fn eat_value(&mut self, x: usize) {
        if let Some(e) = self.v2f_map.get_mut(&x) {
            *e += 1;
        } else {
            self.v2f_map.insert(x, 1);
        }
    }

    /// Builds the sequence of the current order.
    pub fn build_sequence(&mut self) {
        if self.v2f_map.is_empty() {
            self.v2r_maps.push(HashMap::new());
            self.sorted_sequences.push(CompactVector::default());
            return;
        }

        let mut sorted = vec![];
        let mut max_value = 0;

        for (&value, &freq) in &self.v2f_map {
            sorted.push((value, freq));
            max_value = std::cmp::max(max_value, value);
        }
        self.v2f_map.clear();

        // `then_with` is needed to stably sort
        sorted.sort_by(|(v1, f1), (v2, f2)| f2.cmp(f1).then_with(|| v1.cmp(v2)));

        let mut values =
            CompactVector::with_capacity(sorted.len(), needed_bits(max_value)).unwrap();
        sorted
            .iter()
            .for_each(|&(v, _)| values.push_int(v).unwrap());
        self.sorted_sequences.push(values);

        let mut v2r_map = HashMap::new();
        for (i, &(v, _)) in sorted.iter().enumerate() {
            v2r_map.insert(v, i);
        }
        self.v2r_maps.push(v2r_map);
    }

    pub fn rank(&self, order: usize, value: usize) -> Option<usize> {
        self.v2r_maps[order].get(&value).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let seqs = vec![vec![2, 2, 1, 2, 4, 2, 1, 2, 1], vec![2, 1, 2, 1, 1, 1]];

        let mut scb = CountsBuilder::default();
        for seq in &seqs {
            for &x in seq {
                scb.eat_value(x);
            }
            scb.build_sequence();
        }

        assert_eq!(scb.rank(0, 1), Some(1));
        assert_eq!(scb.rank(0, 2), Some(0));
        assert_eq!(scb.rank(0, 3), None);
        assert_eq!(scb.rank(0, 4), Some(2));
        assert_eq!(scb.rank(1, 1), Some(0));
        assert_eq!(scb.rank(1, 2), Some(1));

        let counts = scb.release();
        assert_eq!(counts[0].get_int(0), Some(2));
        assert_eq!(counts[0].get_int(1), Some(1));
        assert_eq!(counts[0].get_int(2), Some(4));
        assert_eq!(counts[1].get_int(0), Some(1));
        assert_eq!(counts[1].get_int(1), Some(2));
    }
}
