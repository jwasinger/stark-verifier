use rustfft::num_traits::Pow;
use blake2::{Blake2s, Digest};
use std::fs::File;
use std::io::Read;
use crate::utils::{as_u32_le};

pub type Value = Vec<u8>;
pub type MerkleDigest = [u8; 32];

#[derive(Default)]
pub struct ProofBranch {
    pub witnesses: Vec<MerkleDigest>,
    pub sibling_value: Value, //should be included as a hash in the list of witnesses but I would have to refactor the python prover merkle tree code
    pub value: Value,
}

#[derive(Default)]
pub struct MultiProof {
    pub branches: Vec<ProofBranch>,
    pub root: MerkleDigest, // TODO remove this field and only allow it to be passed as a parameter to 'verify'
}

// TODO update the package version of this code
impl MultiProof {
    pub fn verify(&self, indices: &[u32], rt: Option<MerkleDigest>) -> Option<Vec<Value>> {
       let mut res: Vec<Value> = Default::default();

       assert!(self.branches.len() == indices.len(), format!("branches len {} != indices len {}", self.branches.len(), indices.len()));

       let root = match rt {
           Some(r) => r.clone(),
           None => self.root.clone()
       };

       for (branch, i) in self.branches.iter().zip(indices.iter()) {
            if let Some(value) = branch.verify(&root, *i)  {
                res.push(value);
            } else {
                return None;
            }
       }

        Some(res) 
    }

    pub fn deserialize(mut f: &File) -> Self {
        let mut branches: MultiProof = Default::default();
        let mut num_branches_bytes = [0u8; 4];
		let mut num_branches: u32 = 0;


		f.read_exact(&mut num_branches_bytes).unwrap();
		num_branches = as_u32_le(&num_branches_bytes);

        let mut branches: Vec<ProofBranch> = Default::default();

        for branch in 0..(num_branches as usize) {
            let mut witnesses_size_bytes = [0u8; 4];
            let mut value_size_bytes = [0u8; 4];

            let mut witnesses: Vec<MerkleDigest> = Default::default();

            f.read_exact(&mut value_size_bytes);
            let value_size = as_u32_le(&mut value_size_bytes);

            let mut value: Value = vec![0u8; value_size as usize];
            let mut sibling_value: Value = vec![0u8; value_size as usize];

            f.read_exact(&mut value[0..value_size as usize]).unwrap();
            f.read_exact(&mut sibling_value[0..value_size as usize]).unwrap();

            f.read_exact(&mut witnesses_size_bytes).unwrap();

            let witnesses_size = as_u32_le(&mut witnesses_size_bytes);
            assert!(witnesses_size % 32 == 0, format!("witnesses should all be 32 bytes: {}", witnesses_size));

            let num_witnesses = witnesses_size / 32;

            for i in 0..(num_witnesses as usize) {
                let mut witness = [0u8; 32];
                f.read_exact(&mut witness);
                witnesses.push(witness);
            }

            branches.push(ProofBranch {
                witnesses: witnesses,
                sibling_value: sibling_value,
                value: value
            });
        }

        let multiproof = MultiProof {
            branches: branches,
            root: Default::default()
        };

        multiproof
    }
}

impl ProofBranch {
    fn permute_4_indices(indices: &Vec<u32>, L: u32) -> Vec<u32> {
        let mut res: Vec<u32> = Vec::new();
        let ld4 = L / 4;
        indices.iter().map(|idx| {
            res.push(Self::permute_4_index(*idx, ld4));
        });

        res
    }

    fn permute_4_index(x: u32, L: u32) -> u32 {
        let ld4 = L / 4;
        let res = (x / ld4) + 4 * (x % ld4);
        res
    }

    // expect the witnesses to be sorted in reverse
    pub fn verify(&self, root: &MerkleDigest, a: u32) -> Option<Value> {
        let idx = Self::permute_4_index(a, 2u32.pow((self.witnesses.len()+1) as u32));

        let mut res = vec![0u8; self.value.len()];
        res[0..self.value.len()].clone_from_slice(&self.value[..]);

        let mut tree_index = 2usize.pow((self.witnesses.len() + 2) as u32) + idx as usize;

        let mut h = Blake2s::default();

        if tree_index % 2 != 0 {
            let b: &[u8] = &res[0..self.value.len()];
            let o: &[u8] = &[&self.sibling_value[..], &b].concat();
            h.input(o);
            res[0..32].clone_from_slice(&h.result()[0..32]);

        } else {
            let b: &[u8] = &res[0..self.value.len()];
            let o: &[u8] = &[&b, &self.sibling_value[..]].concat();

            h.input(o);
            res[0..32].clone_from_slice(&h.result()[0..32]);
        }

        tree_index = tree_index / 2;

        for (i, witness) in self.witnesses.iter().enumerate() {
            let mut hasher = Blake2s::default();

            if tree_index % 2 != 0 {
              let b: &[u8] = &res[0..32];
              let o: &[u8] = &[&witness[..], &b].concat();
              hasher = Default::default();
              hasher.input(o);

              res[0..32].clone_from_slice(&hasher.result()[0..32]);
            } else {
              let b: &[u8] = &res[0..32];
              let o: &[u8] = &[&b, &witness[..]].concat();
              hasher.input(o);
              res[0..32].clone_from_slice(&hasher.result()[0..32]);
            }

            tree_index = tree_index / 2;
        }

        assert!(&res[0..32] == root, format!("values didn't match up {} != {}", hex::encode(&res[0..32]), hex::encode(root)));
        if &res[0..32] == root {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use blake2::{Blake2s, Digest};
    use super::*;

    fn convert(a: &[u8]) -> [u8; 32] {
        let mut b = [0u8; 32];
        b.clone_from_slice(&a[0..32]);
        b
    }

    #[test]
    fn test_verify_single_branch() {
        let idx: u32 = 6997;
        let value: MerkleDigest = convert(&hex::decode("c10b1b74355bf6f5d2085b55bb459f5c14e9e4816c04ec72cca8e8bb5288cb50").unwrap());
        let witnesses: Vec<MerkleDigest> = vec![ convert(&hex::decode("ba5ba7ea7c0fe5ea738a8d207f0797dbb0658bdceeb6be707e5e912f0f7b2728").unwrap()),
            convert(&hex::decode("e845c8789914f8e0b5d11bd7f90471820cfd568e735343bf4661da8b1dda0648").unwrap()),
            convert(&hex::decode("ab163f24c3ba15c5ff71cf5efb50cad7498b82dfa037e436dc05331ef7ebe039").unwrap()),
            convert(&hex::decode("97b87355c6fb49f0993779360f88792d527c572da72bde39305b53de6facda84").unwrap()),
            convert(&hex::decode("37d65601a24040c2faf8c657828174ef9131e0a8c3a30f176fcdeb382a0c2a0d").unwrap()),
            convert(&hex::decode("1325568476a58340b6dd1c0cc2127feff42c8d271aa559802b1b3d93d4899926").unwrap()),
            convert(&hex::decode("3c77b2671efbb23acc796240fc3f4f047bacfc472a3de3d0978540179e4f9e14").unwrap()),
            convert(&hex::decode("92e4884259792a7c891bba4532d25c1dcc74c648553d106344bb10c3840d4c6d").unwrap()),
            convert(&hex::decode("df1a1e30b6fec4aec943937956102149691bc4b06992af48a43694cfa6849165").unwrap()),
            convert(&hex::decode("3cd4cab3eb75edca1054bd5d9673131b834c893ac4997fce779ef295483276a0").unwrap()),
            convert(&hex::decode("04126daaacf18b81a8008309feebb894e58a9bd3051ba8951a90f51e7d46a99f").unwrap()),
            convert(&hex::decode("9e0cf8e9dac36bd52ce3cad4eb4fc0bf930e7543e5c790d61f6bd5c9f85ca0ee").unwrap()),
            convert(&hex::decode("4c9649fbaad59383acc8dc17d6a2b47408e081b7ff955e983580536a2dbc3fd7").unwrap()),
            convert(&hex::decode("89b6f049e518dc76c69f797ce4c7bc9c1b45328ea957b2634f21e7a10af9b0cb").unwrap())];

        let root: MerkleDigest = convert(&hex::decode("f13a4bfaa28c22df47a4e0e89a54736b0a9bedc9727bbc3cc8a0e4237eb59ad9").unwrap());

        let proof_branch = ProofBranch {
            witnesses,
            value
        };

        assert!(proof_branch.verify(&root, root).is_some(), "proof was invalid");
    }
}
