use yada::{builder::DoubleArrayBuilder, DoubleArray};

#[derive(Default, Debug)]
pub struct Vocabulary {
    data: Vec<u8>,
}

impl Vocabulary {
    pub fn new(tokens: &[&str]) -> Self {
        let mut keyset: Vec<(&str, u32)> = tokens
            .iter()
            .enumerate()
            .map(|(id, token)| (*token, id as u32))
            .collect();
        keyset.sort_by(|(t1, _), (t2, _)| t1.cmp(t2));

        Self {
            data: DoubleArrayBuilder::build(&keyset).unwrap(),
        }
    }

    pub fn get(&self, token: &str) -> Option<usize> {
        let da = DoubleArray::new(&self.data[..]);
        da.exact_match_search(token).map(|x| x as usize)
    }
}
