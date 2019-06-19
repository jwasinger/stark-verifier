use num_bigint::BigUint;
use rustfft::num_traits::Pow;

pub type Digest = [u8; 32];
pub struct Witnesses(Vec<Digest>);

impl Witnesses {
    pub fn verify_branch(&self, root: &[u8; 32], idx: u32) -> bool {
        let index = BigUint::from(idx) + BigUint::from(2u8).pow(self.0.len() * 32);
        println!("index is {}", index);
        
        false

    }

    pub fn verify_multi_branch(&self, root: &[u8; 32], indices: Vec<usize>) -> bool {
        assert!(indices.len() == self.0.len(), "invalid indices len or self len");

        //let partial_tree: HashMap<usize, Digest>

        for i in 0..indices.len() {
            let proof_branch = &self.0[i];
            let half_tree_size: usize = 2usize.pow(proof_branch.len() as u32 -1);
            let index = half_tree_size + i;
        }

        /*
        for (i, b) in &indices.iter().zip(&self.0).collect() {
            println!("{}", &i);
            println!("{}", &b);
        }
        */

        return true;
    }

    fn deserialize(bytes: &[u8], witnesses_len: u8) -> Option<Self> {
        assert!(witnesses_len as usize * 32 == bytes[8..].len(), "bad length for witnesses");

        let mut res: Vec<Digest> = Vec::with_capacity(witnesses_len as usize);

        for i in (0..bytes[8..].len()).step_by(32) {
            let mut digest = [0u8; 32];
            digest.clone_from_slice(&bytes[i..i+32]);

        }

        return Some(Witnesses {
            0: res
        });
    }

    pub fn deserialize_multi(bytes: &[u8]) -> Option<(Vec<Self>, usize)> {
        // each branch is 17 (possibly empty) digest elements

        let mut idx = 0;

        let mut res: Vec<Witnesses> = Vec::new();

        while idx < bytes.len() {
            let witnesses_len = bytes[idx];
            assert!(witnesses_len < 15, "too many witnesses!");
            assert!(bytes.len() - idx >= witnesses_len as usize * 32, "incorrect witness size or missing witness data");

            res.push(Self::deserialize(&bytes[idx..witnesses_len as usize * 32], witnesses_len)?);
            idx += 8 + witnesses_len as usize * 32;
        }

        Some((res, idx))
    }
}
