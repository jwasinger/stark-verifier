use std::mem::{size_of};

const PREAMBLE_SIZE: usize = 64;

pub struct StarkProof {
    pub merkle_root: [u8; 32],
    pub l_merkle_root: [u8; 32],
    pub fri_proof: Vec<LowDegreeProofElement>,
    pub merkle_branches: Vec<u8>, // TODO
    pub linear_comb_branches: Vec<u8>, // TODO
}

pub struct LowDegreeProofElement {
    pub root2: [u8; 32],
    pub column_branches: Vec<u8>, // TODO
    pub poly_branches: Vec<u8>  // TODO
}

impl StarkProof {
    /*
    Data Format

    */
    pub fn deserialize(data: &Vec<u8>) -> Result<Self, &'static str> {
        let low_degree_proof_size: usize  = data.len() - PREAMBLE_SIZE; // TODO how to make this const/

        if data.len() == 0 {
            return Err(From::from("preamble section was bad"));
        }

        if data.len() % size_of::<LowDegreeProofElement>() != 0 {
            return Err(From::from("low degree proof elements incorrect size"));
        }

        let num_low_degree_proof_elems = low_degree_proof_size % size_of::<LowDegreeProofElement>();

        //let merkle_root = data[0..PREAMBLE_SIZE-32];
        //let l_merkle_root = data[(PREAMBLE_SIZE-32)..PREAMBLE_SIZE];

        /*
        for i in 0..num_low_degree_proof_elems {
            output.low_degree_proof.push(LowDegreeProofElement {
              root2

            })
        }
        */

        let mut merkle_root = [0u8; 32];
        let mut l_merkle_root = [0u8; 32];
        let mut low_degree_proof: Vec<LowDegreeProofElement> = Vec::new();

        merkle_root.clone_from_slice(&data[0..PREAMBLE_SIZE-32]);
        l_merkle_root.clone_from_slice(&data[(PREAMBLE_SIZE-32)..PREAMBLE_SIZE]);

        let mut low_degree_proof = Self {
            merkle_root, 
            l_merkle_root,
            merkle_branches: Vec::new(), //TODO ignoring merkle_branches and linear_comb_branches for now
            linear_comb_branches: Vec::new(),
            fri_proof: Vec::new(),
        };

        let ldp_offset = PREAMBLE_SIZE;

        /*
        TODO
        for ldp_offset in (PREAMBLE_SIZE..data.len()).step_by(size_of::<LowDegreeProofElement>()) {
        //for (ldp_offset < data.len() - mem::size_of(LowDegreeProofElement); ldp_offset += mem::size_of(LowDegreeProofElement)) {
            let mut root2 = [0u8; 32];
            let mut column_branches = [0u8; 96];
            let mut poly_branches = [0u8; 32];
            root2.clone_from_slice(&data[ldp_offset..96]);
            //column_branches.clone_from_slice(&data[ldp_offset+96..ldp_offset+96+96]);
            //poly_branches.clone_from_slice(&data[ldp_offset+96+96..ldp_offset+96+96+32]);

            low_degree_proof.fri_proof.push(LowDegreeProofElement{
                root2,
                column_branches,
                poly_branches
            });
        }
        */

        low_degree_proof.fri_proof = Vec::new();

        Ok(low_degree_proof)
    }
}
