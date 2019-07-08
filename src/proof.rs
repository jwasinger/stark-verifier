use std::mem::{size_of};
use crate::merkle_tree::{MultiProof, MerkleDigest};
use crate::utils::{as_u32_le};
use std::fs::{File};
use std::io::Read;

// use crate::merkle_tree::{Witnesses};

const PREAMBLE_SIZE: usize = 64;

// Proof element type constants (TODO make these less hacky)
const MERKLE: u32 = 1;
const POINTS: u32 = 2;

pub struct FRIProof {
    pub merkle_proofs: Vec<LDPMerkleProof>,
    pub points_proof: LDPPointsProof,
}

pub type LDPPointsProof = Vec<u32>;

pub struct StarkProof {
    pub merkle_root: MerkleDigest,
    pub l_merkle_root: MerkleDigest,
    pub fri_proof: FRIProof,
    pub merkle_branches: Vec<u8>, // TODO
    pub linear_comb_branches: Vec<u8>, // TODO
}

#[derive(Default)]
pub struct LDPMerkleProof {
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
        let mut next_offset: usize = 0;

        //println!("about to read stuff");
        file.read_exact(&mut merkle_root).unwrap();
        file.read_exact(&mut l_merkle_root).unwrap();

        //println!("merkle root is {:x?}", &merkle_root);
        //println!("merkle root 2 is {:x?}", &l_merkle_root);

        //let mut num_ldp_elements_bytes = [0u8; 4];
        //file.read_exact(&mut num_ldp_elements_bytes).unwrap();
        //let num_ldp_elements = as_u32_le(&num_ldp_elements_bytes);

        /* a low degree proof consists of:
             a merkle root
             a MultiProof for column points
             a MultiProof for poly points
        */

        //println!("{}", num_ldp_elements);
        //for i in 0..(num_ldp_elements as usize) {

        let mut ldp_merkle_proofs: Vec<LDPMerkleProof> = Vec::new();
        let mut points_proof: LDPPointsProof = Default::default();
        let mut done = false;
        while !done {
            // read the type
            let mut proof_element_type_bytes = [0u8; 4];
            file.read_exact(&mut proof_element_type_bytes).unwrap();

            //println!("proof element bytes {:x?}", proof_element_type_bytes);
            let proof_element_type = as_u32_le(&proof_element_type_bytes);

            if proof_element_type == MERKLE {
                let mut m: LDPMerkleProof = Default::default();
                file.read_exact(&mut m.root2).unwrap();

                //println!("MERKLE");
                m.column_branches = MultiProof::deserialize(&mut file);
                m.poly_branches = MultiProof::deserialize(&mut file);
                ldp_merkle_proofs.push(m);
            } else if proof_element_type == POINTS {
                let mut points_size_bytes = [0u8; 4];
                file.read_exact(&mut points_size_bytes).unwrap();
                //println!("read points");
                //println!("POINTS");

                let points_size = as_u32_le(&points_size_bytes);
                assert!(points_size > 0, "more than zero points required");
                assert!(points_size % 32 == 0, "points size not divisible by 32");
                //println!("number of points is {}", points_size);

                let mut points_bytes = vec![0u8; points_size as usize];
                file.read_exact(&mut points_bytes);

                done = true;
            } else {
                panic!("invalid proof element type");
            }
        }

        let fri_proof = FRIProof {
            merkle_proofs: ldp_merkle_proofs,
            points_proof: points_proof,
        };

        // merkle_root.clone_from_slice(&data[0..PREAMBLE_SIZE-32]);
        // l_merkle_root.clone_from_slice(&data[(PREAMBLE_SIZE-32)..PREAMBLE_SIZE]);

        // let (merkle_branches, next_offset) = Witnesses::deserialize_multi(&data[PREAMBLE_SIZE..data.len()]).expect("merkle branches malformed");
        // let (linear_comb_branches, next_offset) = Witnesses::deserialize_multi(&data[PREAMBLE_SIZE+next_offset..data.len()]).expect("linear combination malformed");

        /*
        let (fri_proof, next_offset) = Witnesses::deserialize_multi(&data[PREAMBLE_SIZE..]).expect("linear combination malformed");
        */


        let mut proof = Self {
            merkle_root, 
            l_merkle_root,
            merkle_branches: Vec::new(), //TODO ignoring merkle_branches and linear_comb_branches for now
            linear_comb_branches: Vec::new(),
            fri_proof: fri_proof,
        };

        Ok(proof)
    }
}
