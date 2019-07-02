use num_bigint::BigUint;
use rustfft::num_traits::Pow;
use blake2::{Blake2s, Digest};

type Value = [u8; 32];
type MerkleDigest = [u8; 32];

pub struct ProofBranch {
    witnesses: Vec<MerkleDigest>,
    value: [u8; 32],
    idx: u32,
}

pub struct MultiProof {
    branches: Vec<ProofBranch>,
    root: MerkleDigest,
}

impl MultiProof {
    pub fn verify(&self) -> Option<Vec<Value>> {
       let mut res: Vec<Value> = Default::default();

       for branch in self.branches.iter() {
            if let Some(value) = branch.verify(&self.root)  {
                res.push(value);
            } else {
                return None;
            }
       }

        Some(res) 
    }
}

impl ProofBranch {
    // expect the witnesses to be sorted in reverse
    pub fn verify(&self, root: &MerkleDigest) -> Option<[u8; 32]> {

        //hasher.input(&self.value);

        let mut res: MerkleDigest = [0u8; 32];
        //assert!(hasher.result().len() == 32, "invalid digest result");
        res[0..32].clone_from_slice(&self.value);

        //let mut res = blake2::hash(proof.value);
        let mut tree_index = 2usize.pow((self.witnesses.len() + 1) as u32) + self.idx as usize;
        // assert!(self.witnesses.len() == 256, "invalid proof length");

        for (i, witness) in self.witnesses.iter().enumerate() {
            let mut hasher = Blake2s::default();
            println!("tree_index is {}", tree_index);

            if tree_index % 2 != 0 {
              println!("left");

              let b: &[u8] = &res[0..32];
              let o: &[u8] = &[&witness[..], &b].concat();
              hasher = Default::default();
              hasher.input(o);
              println!("witness part is {}", hex::encode(&witness[..]));
              println!("computed part is {}", hex::encode(&b));
              println!("input is {}", hex::encode(o));
              //res = hasher.result();
              res[0..32].clone_from_slice(&hasher.result()[0..32]);
            } else {
              println!("right");
              let b: &[u8] = &res[0..32];
              let o: &[u8] = &[&b, &witness[..]].concat();
              println!("witness part is {}", hex::encode(&witness[..]));
              println!("computed part is {}", hex::encode(&b));
              println!("input is {}", hex::encode(o));
              hasher.input(o);
              //res = hasher.result();
              res[0..32].clone_from_slice(&hasher.result()[0..32]);
            }

            tree_index = tree_index / 2;

            println!("res is {}", hex::encode(&res[0..32]));
        }

        //assert!(&res[0..32] == &self.root, format!("values didn't match up {} != {}", hex::encode(&res[0..32]), hex::encode(&self.root)));
        if &res == root {
            Some(res)
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
            value,
            idx
        };

        assert!(proof_branch.verify(&root).is_some(), "proof was invalid");
    }
}
