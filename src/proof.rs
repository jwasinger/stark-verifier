use std::mem::{size_of};
use crate::merkle_tree::{MultiProof, MerkleDigest};
use crate::utils::{as_u32_le};
use std::fs::{File};
use std::io::Read;

pub struct FRIProof {
    pub merkle_proofs: Vec<LDPMerkleProof>,
    pub points_proof: LDPPointsProof,
}

pub type LDPPointsProof = Vec<u32>;

pub struct StarkProof {
    pub merkle_root: MerkleDigest,
    pub l_merkle_root: MerkleDigest,
    pub fri_proof: FRIProof,
    pub merkle_branches: MultiProof,
    pub linear_comb_branches: MultiProof,
}

#[derive(Default)]
pub struct LDPMerkleProof {
    pub root2: [u8; 32],
    pub column_branches: MultiProof,
    pub poly_branches: MultiProof
}
