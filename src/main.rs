pub mod utils;
pub mod proof;
pub mod merkle_tree;
pub mod fft;
pub mod deserializer;

#[macro_use]
extern crate lazy_static;

use num_bigint::{BigInt, BigUint};
use num_bigint::Sign;
use rustfft::num_traits::Pow;
use std::str::FromStr;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::time::{Duration, Instant};

use blake2::{Blake2b, Blake2s, Digest};
use std::mem::transmute;
use self::proof::{StarkProof, LDPMerkleProof, LDPPointsProof};
use merkle_tree::MultiProof;

use crate::proof::FRIProof;
use crate::utils::{is_power_of_2, get_pseudorandom_indices, mimc, as_u32_le, multi_interp_4, eval_quartic, divmod, eval_poly_at, lagrange_interp_2, mul_polys, negative_to_positive};
use crate::fft::fft_inv;
use rustfft::num_traits::sign::Signed;
use rustfft::num_traits::identities::{One, Zero};

const EXTENSION_FACTOR: usize = 8;
const MODULUS: &str = "115792089237316195423570985008687907853269984665640564039457584006405596119041";

fn verify_low_degree_proof(merkle_root: &[u8; 32], mut root_of_unity: BigInt, proof: &FRIProof,  mut max_deg_plus_1: BigInt, modulus: &BigInt, excludeMultiplesOf: Option<u32>) -> bool {
    let mut test_val = root_of_unity.clone(); 
    let mut rou_deg: usize = 1;
    let mut root = merkle_root;

    while test_val != BigInt::from(1u32) {
        rou_deg = rou_deg * 2;
        test_val = test_val.modpow(&BigInt::from(2u8), &modulus).clone();
    }
    
    let mut arg: usize = 0; 

    let mut quartic_roots_of_unity: [BigInt; 4] = [
        BigInt::from(1u32),
        root_of_unity.modpow(&BigInt::from(rou_deg / 4), &modulus),
        root_of_unity.modpow(&BigInt::from(rou_deg / 2), &modulus),
        root_of_unity.modpow(&BigInt::from(rou_deg * 3 / 4), &modulus)
    ];

    assert!(&rou_deg == &65536usize, "invalid roudeg");
    assert!(&quartic_roots_of_unity[3] == &FromStr::from_str("80127877722526290441229381276271393407378829608771736609433200039324583025757").unwrap(), "bad quartic roots of unity..");

    for m_proof in &proof.merkle_proofs {
        let special_x = BigInt::from_bytes_be(Sign::Plus, root);

        let ys = get_pseudorandom_indices(&m_proof.root2, 40, (rou_deg / 4) as u32, excludeMultiplesOf);

        let column_values = m_proof.column_branches.verify(&ys, None).unwrap();

        let mut poly_positions: Vec<u32> = Vec::new();

        for y in &ys {
            for i in 0..4 {
                poly_positions.push(y + ( (rou_deg as u32) / 4) * i); 
            }
        }

        let poly_values = m_proof.poly_branches.verify(&poly_positions, Some(root.clone())).unwrap();

        let mut xcoords: Vec<BigInt> = Vec::new();
        let mut rows: Vec<BigInt> = Vec::new();

        for (i, y) in (&ys).iter().enumerate() {
            let x1 = root_of_unity.modpow(&BigInt::from(*y), &modulus);

            for j in 0..4 {
                xcoords.push(((&quartic_roots_of_unity[j]) * &x1) % modulus);
                rows.push(BigInt::from_bytes_be(Sign::Plus, &poly_values[i*4 + j]));
            }
        }

        let polys: Vec<BigInt> = multi_interp_4(&xcoords, &rows, modulus);

        for (p, c) in polys.chunks(4).zip(column_values.iter()) {
            assert!(eval_quartic(&p, &special_x, modulus) == BigInt::from_bytes_be(Sign::Plus, c), "low degree test failed...");
        }

        root_of_unity = root_of_unity.modpow(&BigInt::from(4u8), &modulus); 
        max_deg_plus_1 = max_deg_plus_1 / BigInt::from(4u8);
        rou_deg = rou_deg / 4;
        root = &m_proof.root2;
    }

    // TODO direct verification of the low degree proof components

    true
}

fn verify_mimc_proof(inp: BigInt, num_steps: usize, round_constants: &Vec<BigInt>, output: BigInt, proof: StarkProof, modulus: &BigInt) -> bool {

    if num_steps > (2usize.pow(32) / EXTENSION_FACTOR) { //TODO use of floor here?
        return false;
    }

    if !is_power_of_2(num_steps as u32) || !is_power_of_2(round_constants.len() as u32) {
        return false;
    }

    if (round_constants.len() as u32) > num_steps as u32 {
        return false;
    }

    let precision = num_steps * EXTENSION_FACTOR;
    let G2: BigInt = BigInt::from(7u32).modpow(&((modulus.clone() - BigInt::from(1u32)) / precision), &modulus); // TODO do I need floor() here for some reason?
    let skips = precision / num_steps;
    let skips2 = num_steps / round_constants.len();

    let val = G2.modpow(&BigInt::from(EXTENSION_FACTOR*skips2), &modulus);

    assert!(num_steps == 8192, "num steps incorrect");
    assert!(G2 == FromStr::from_str("41913712888260089065520476180880993127517355946012995597287997778376518235852").unwrap(), "G2 isn't correct");

    assert!(val == FromStr::from_str("56670364103764250102176604807203318908867195832872336813161821519223575486477").unwrap(), "constants mini polynomial root wasn't correct");

    let constants_mini_polynomial = fft_inv(round_constants, &val, &modulus);

    if !verify_low_degree_proof(&proof.l_merkle_root, G2.clone(), &proof.fri_proof, BigInt::from(num_steps * 2), &modulus, Some(EXTENSION_FACTOR as u32)) {
        return false;
    }

    let mut hasher = Blake2s::default();

    hasher.input(&[&proof.merkle_root[..], &[1u8]].concat());
    let k1 = BigInt::from_bytes_be(Sign::Plus, &hasher.result());

    hasher = Blake2s::default();
    hasher.input(&[&proof.merkle_root[..], &[2u8]].concat());
    let k2 = BigInt::from_bytes_be(Sign::Plus, &hasher.result());

    hasher = Blake2s::default();
    hasher.input(&[&proof.merkle_root[..], &[3u8]].concat());
    let k3 = BigInt::from_bytes_be(Sign::Plus, &hasher.result());

    hasher = Blake2s::default();
    hasher.input(&[&proof.merkle_root[..], &[4u8]].concat());
    let k4 = BigInt::from_bytes_be(Sign::Plus, &hasher.result());

    let samples: u32 = 80; // spot check security factor
    let positions = get_pseudorandom_indices(&proof.l_merkle_root, samples as usize, precision as u32, Some(EXTENSION_FACTOR as u32));

    let mut augmented_positions: Vec<u32> = Vec::new();

    for p in &positions {
        augmented_positions.push(*p);
        augmented_positions.push((*p + skips as u32) % precision as u32);
    }

    let values = proof.merkle_branches.verify(&augmented_positions, Some(proof.merkle_root.clone())).unwrap();
    let linear_comb_values = proof.linear_comb_branches.verify(&positions, Some(proof.l_merkle_root.clone())).unwrap();

    let last_step_position = G2.modpow(&BigInt::from((num_steps - 1) * skips), modulus);

    for (i, p) in positions.iter().enumerate() {
        let x = G2.modpow(&BigInt::from(*p), modulus);
        let n_steps = BigInt::from(num_steps);
        let x_to_the_steps = x.modpow(&n_steps, modulus);
        let m_branch_1 = &values[i*2];
        let m_branch_2 = &values[i*2 + 1];
        let l_of_x = BigInt::from_bytes_be(Sign::Plus, &linear_comb_values[i]);

        let p_of_x = BigInt::from_bytes_be(Sign::Plus, &m_branch_1[0..32]);
        let p_of_g1x = BigInt::from_bytes_be(Sign::Plus, &m_branch_2[0..32]);
        let d_of_x = BigInt::from_bytes_be(Sign::Plus, &m_branch_1[32..64]);
        let b_of_x = BigInt::from_bytes_be(Sign::Plus, &m_branch_1[64..96]);

        let z_value = divmod(&(&x.modpow(&n_steps, &modulus) - BigInt::one()), &(&x - &last_step_position), &modulus);

        let k_of_x = eval_poly_at(&constants_mini_polynomial, &x.modpow(&BigInt::from(skips2), modulus), modulus);

        // Check transition constraints C(P(x)) = Z(x) * D(x)
        assert!((&p_of_g1x - &p_of_x.pow(3u32) - &k_of_x - &z_value * &d_of_x) % modulus == BigInt::zero(), "invalid proof: transition constraints check failed");

        //Check boundary constraints B(x) * Q(x) + I(x) = P(x)
        let interpolant = lagrange_interp_2(&[BigInt::one(), last_step_position.clone()], &[inp.clone(), output.clone()], modulus);	
        let zeropoly2 = mul_polys(&vec![-BigInt::one(), BigInt::one()], &vec![-last_step_position.clone(), BigInt::one()], modulus);

        assert!(negative_to_positive(&(&p_of_x - &b_of_x * eval_poly_at(&zeropoly2, &x, modulus) - eval_poly_at(&interpolant.to_vec(), &x, modulus)), modulus) == BigInt::zero(), "invalid proof: boundary constraints B(x) * Q(x) + I(x) = P(x) check failed");
        
        // Check correctness of the linear combination
        assert!(negative_to_positive(&(&l_of_x - &d_of_x - &k1 * &p_of_x - &k2 * &p_of_x * &x_to_the_steps - 
            &k3 * &b_of_x - &k4 * &b_of_x * &x_to_the_steps), modulus) == BigInt::zero(), "invalid linear combination");
    }

    println!("proof verified");

    return true;
}

fn main() {
    let mut file = File::open("proof.bin").unwrap();
    let mut file_bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut file_bytes);

    let (proof, _) = deserializer::from_bytes(&file_bytes).expect("couldn't deserialize"); 
    const LOG_STEPS: usize = 13;
    let mut constants: Vec<BigInt> = Vec::new();
    let modulus: BigInt = BigInt::from_str(MODULUS).expect("modulus couldn't be deserialized into bigint");

    for i in 0..64 {
        let constant = BigInt::from(i as u8).pow(BigUint::from(7u8)) ^ BigInt::from(42u8);
        constants.push(constant);
    }

    let mimc_time = Instant::now();
    let output = mimc(&BigInt::from(3u8), 2usize.pow(LOG_STEPS as u32), &constants, &modulus);

    println!("took {:?} to compute {} rounds of mimc", mimc_time.elapsed(), 2usize.pow(LOG_STEPS as u32));
    println!("output is {}", &output);
    
    let stark_time = Instant::now();
    // TODO start measuring for benchmarks here
    if !verify_mimc_proof(BigInt::from(3u8), 2usize.pow(LOG_STEPS as u32), &constants, output, proof, &modulus) {
        panic!("could not verify mimc stark proof");
    }

    println!("took {:?} to verify stark proof", stark_time.elapsed());
}
