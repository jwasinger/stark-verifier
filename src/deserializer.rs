use crate::merkle_tree::*;
use crate::proof::*;
use crate::utils::*;

// FRI proof element types
const MERKLE: u32 = 1;
const POINTS: u32 = 2;

fn read_exact(source: &[u8], destination: &mut [u8], offset: u32) -> Result<u32, &'static str> {
    println!("offset is {}", offset);
    let len = destination.len();
    destination[..].clone_from_slice(&source[(offset as usize)..(offset as usize + len)]);
    Ok(len as u32)
}

fn _stark_proof_from_bytes(bytes: &[u8], start_offset: u32) -> Result<(StarkProof, u32), &'static str> {
    let mut offset = start_offset;
    let mut merkle_root = [0u8; 32];
    let mut l_merkle_root = [0u8; 32];

    println!("intput len is {}", bytes.len());
    offset += read_exact(bytes, &mut merkle_root[..], offset).unwrap();
    offset += read_exact(bytes, &mut l_merkle_root[..], offset).unwrap();

    let mut ldp_merkle_proofs: Vec<LDPMerkleProof> = Vec::new();
    let mut points_proof: LDPPointsProof = Default::default();
    let mut done = false;
    while !done {
        // read the type
        let mut proof_element_type_bytes = [0u8; 4];
        offset += read_exact(bytes, &mut proof_element_type_bytes[..], offset).unwrap();

        let proof_element_type = as_u32_le(&proof_element_type_bytes);

        if proof_element_type == MERKLE {
            let mut m: LDPMerkleProof = Default::default();
            offset += read_exact(bytes, &mut m.root2[..], offset).unwrap();

            let (column_branches, column_branches_size) = bytes_to_multiproof(bytes, offset).expect("column branches deserialization");
            m.column_branches = column_branches;
            offset += column_branches_size;
            m.column_branches.root = m.root2.clone(); // TODO replace this by having branches reference the same merkle root instead of copying it for each proof
            let (poly_branches, poly_branches_size) =  bytes_to_multiproof(bytes, offset).expect("poly branches deserialization");
            m.poly_branches = poly_branches;
            ldp_merkle_proofs.push(m);
            offset += poly_branches_size;
        } else if proof_element_type == POINTS {
            // points are the direct component of the stark proof
            let mut points_size_bytes = [0u8; 4];
            offset += read_exact(bytes, &mut points_size_bytes[..], offset).unwrap();

            let points_size = as_u32_le(&points_size_bytes);
            assert!(points_size > 0, "more than zero points required");
            assert!(points_size % 32 == 0, "points size not divisible by 32");

            let mut points_bytes = vec![0u8; points_size as usize];
            offset += read_exact(bytes, &mut points_bytes[..], offset).unwrap();

            done = true;
        } else {
            panic!("invalid proof element type");
        }
    }

    let fri_proof = FRIProof {
        merkle_proofs: ldp_merkle_proofs,
        points_proof: points_proof,
    };

    let (merkle_branches, mut size) = bytes_to_multiproof(bytes, offset).expect("main merkle branches");
    offset += size as u32;

    let (linear_comb_branches, size) = bytes_to_multiproof(bytes, offset).expect("linear combination branches");
    offset += size as u32;

    let mut proof = StarkProof {
        merkle_root, 
        l_merkle_root,
        merkle_branches: merkle_branches,
        linear_comb_branches: linear_comb_branches,
        fri_proof: fri_proof,
    };

    Ok((proof, offset))
}

fn bytes_to_multiproof(bytes: &[u8], start_offset: u32) -> Result<(MultiProof, u32), &'static str> {
    let mut branches: MultiProof = Default::default();
    let mut num_branches_bytes = [0u8; 4];
    let mut num_branches: u32 = 0;
    let mut offset = start_offset;

    offset += read_exact(bytes, &mut num_branches_bytes[..], offset).unwrap();
    num_branches = as_u32_le(&num_branches_bytes);

    let mut branches: Vec<ProofBranch> = Default::default();

    for branch in 0..(num_branches as usize) {
        let mut witnesses_size_bytes = [0u8; 4];
        let mut value_size_bytes = [0u8; 4];

        let mut witnesses: Vec<MerkleDigest> = Default::default();

        offset += read_exact(bytes, &mut value_size_bytes[..], offset).unwrap();

        let value_size = as_u32_le(&mut value_size_bytes);

        let mut value: Value = vec![0u8; value_size as usize];
        let mut sibling_value: Value = vec![0u8; value_size as usize];

        offset += read_exact(bytes, &mut value[0..value_size as usize], offset).unwrap();
        offset += read_exact(bytes, &mut sibling_value[0..value_size as usize], offset).unwrap();

        offset += read_exact(bytes, &mut witnesses_size_bytes[..], offset).unwrap();

        let witnesses_size = as_u32_le(&mut witnesses_size_bytes);
        assert!(witnesses_size % 32 == 0, format!("witnesses should all be 32 bytes: {}", witnesses_size));

        let num_witnesses = witnesses_size / 32;

        for i in 0..(num_witnesses as usize) {
            let mut witness = [0u8; 32];
            offset += read_exact(bytes, &mut witness[..], offset).unwrap();
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

    Ok((multiproof, offset - start_offset))
}

pub fn from_bytes(bytes: &[u8]) -> Result<(StarkProof, u32), &'static str> {
    _stark_proof_from_bytes(bytes, 0)
}
