use std::mem::{size_of};
use crate::merkle_tree::{MultiProof, MerkleDigest};
use std::fs::{File};
use std::io::Read;

// use crate::merkle_tree::{Witnesses};

const PREAMBLE_SIZE: usize = 64;

pub struct StarkProof {
    pub merkle_root: MerkleDigest,
    pub l_merkle_root: MerkleDigest,
    pub fri_proof: Vec<LowDegreeProofElement>,
    pub merkle_branches: Vec<u8>, // TODO
    pub linear_comb_branches: Vec<u8>, // TODO
}

#[derive(Default)]
pub struct LowDegreeProofElement {
    pub root2: [u8; 32],
    pub column_branches: MultiProof, // TODO
    pub poly_branches: MultiProof  // TODO
}

impl StarkProof {
    /*
    Data Format

    */
    pub fn deserialize(mut file: File) -> Result<Self, &'static str> {
        let mut merkle_root = [0u8; 32];
        let mut l_merkle_root = [0u8; 32];
        let mut low_degree_proof: Vec<LowDegreeProofElement> = Vec::new();
        let mut next_offset: usize = 0;

        file.read_exact(&mut merkle_root).unwrap();
        file.read_exact(&mut l_merkle_root).unwrap();

        let mut num_ldp_elements = [0u8];
        
        file.read_exact(&mut num_ldp_elements).unwrap();

        /* a low degree proof consists of:
             a merkle root
             a MultiProof for column points
             a MultiProof for poly points
        */

        let res: Vec<LowDegreeProofElement> = Default::default();
        
        for i in 0..(num_ldp_elements[0] as usize) {
            let mut element: LowDegreeProofElement = Default::default();
            file.read_exact(&mut element.root2).unwrap();

            let column_branches = MultiProof::deserialize(&mut file);
        }

        // merkle_root.clone_from_slice(&data[0..PREAMBLE_SIZE-32]);
        // l_merkle_root.clone_from_slice(&data[(PREAMBLE_SIZE-32)..PREAMBLE_SIZE]);

        // let (merkle_branches, next_offset) = Witnesses::deserialize_multi(&data[PREAMBLE_SIZE..data.len()]).expect("merkle branches malformed");
        // let (linear_comb_branches, next_offset) = Witnesses::deserialize_multi(&data[PREAMBLE_SIZE+next_offset..data.len()]).expect("linear combination malformed");

        /*
        let (fri_proof, next_offset) = Witnesses::deserialize_multi(&data[PREAMBLE_SIZE..]).expect("linear combination malformed");
        */


        let mut low_degree_proof = Self {
            merkle_root, 
            l_merkle_root,
            merkle_branches: Vec::new(), //TODO ignoring merkle_branches and linear_comb_branches for now
            linear_comb_branches: Vec::new(),
            fri_proof: Vec::new(),
        };

        let ldp_offset = PREAMBLE_SIZE;

        low_degree_proof.fri_proof = Vec::new();

        Ok(low_degree_proof)
    }
}
