  /*
Proof:
    merkle roots of P, D, spot check merkle proofs, low-degree proofs of P and D
*/


pub mod utils;
pub mod proof;
pub mod merkle_tree;
pub mod fft;

#[macro_use]
extern crate lazy_static;

use num_bigint::BigUint;
use rustfft::num_traits::Pow;
use std::str::FromStr;

use std::fs::File;
use std::io;
use std::io::prelude::*;

use blake2::{Blake2b, Digest};
use std::mem::transmute;
use self::proof::{StarkProof, LDPMerkleProof, LDPPointsProof};
use merkle_tree::MultiProof;

use crate::proof::FRIProof;
use crate::utils::{is_power_of_2, get_pseudorandom_indices, mimc, as_u32_le};
use crate::fft::fft_inv;

const EXTENSION_FACTOR: usize = 8;
const MODULUS: &str = "115792089237316195423570985008687907853269984665640564039457584006405596119041";

fn verify_low_degree_proof(merkle_root: &[u8; 32], root_of_unity: &BigUint, proof: &FRIProof,  max_deg_plus_1: &BigUint, modulus: &BigUint, excludeMultiplesOf: Option<u32>) -> bool {
    //println!("verifying low degree proof");
    let mut test_val = root_of_unity.clone(); 
    let mut rou_deg: usize = 1;

    while test_val != BigUint::from(1u32) {
        rou_deg = rou_deg * 2;
        test_val = test_val.modpow(&BigUint::from(2u8), &modulus).clone();
    }
    
    let mut arg: usize = 0; 

    let mut quartic_roots_of_unity: [BigUint; 4] = [
        BigUint::from(1u32),
        root_of_unity.modpow(&BigUint::from(rou_deg / 4), &modulus),
        root_of_unity.modpow(&BigUint::from(rou_deg / 2), &modulus),
        root_of_unity.modpow(&BigUint::from(rou_deg * 3 / 4), &modulus)
    ];

    assert!(&rou_deg == &65536usize, "invalid roudeg");
    assert!(&quartic_roots_of_unity[3] == &FromStr::from_str("80127877722526290441229381276271393407378829608771736609433200039324583025757").unwrap(), "bad quartic roots of unity..");

    for m_proof in &proof.merkle_proofs {
        //let (root2, column_branches, poly_branches) = p;
        let special_x = BigUint::from_bytes_be(merkle_root);

        //println!("root 2 is {:x?}", &m_proof.root2);
        //println!("rou_deg / 4 = {}", &rou_deg / 4);


        let ys = get_pseudorandom_indices(&m_proof.root2, 40, (rou_deg / 4) as u32, excludeMultiplesOf);

        m_proof.column_branches.verify(&ys[1..]).unwrap();

        // let poly_positions = sum([[y + (roudeg // 4) * j for j in range(4)] for y in ys], []);

        println!("ys: ");
        for y in ys {
            println!("{}", &y);
        }
    }

    // verify the polynomial

    true
}

fn verify_mimc_proof(inp: BigUint, num_steps: usize, round_constants: &Vec<BigUint>, output: BigUint, proof: StarkProof, modulus: &BigUint) -> bool {

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
    let G2: BigUint = BigUint::from(7u32).modpow(&((modulus.clone() - BigUint::from(1u32)) / precision), &modulus); // TODO do I need floor() here for some reason?
    let skips = precision / num_steps;
    let skips2 = num_steps / round_constants.len();

    let val = G2.modpow(&BigUint::from(EXTENSION_FACTOR*skips2), &modulus);

    assert!(num_steps == 8192, "num steps incorrect");
    assert!(G2 == FromStr::from_str("41913712888260089065520476180880993127517355946012995597287997778376518235852").unwrap(), "G2 isn't correct");

    assert!(val == FromStr::from_str("56670364103764250102176604807203318908867195832872336813161821519223575486477").unwrap(), "constants mini polynomial root wasn't correct");

    let constants_mini_polynomial = fft_inv(round_constants, &val, &modulus);
    assert!(constants_mini_polynomial[constants_mini_polynomial.len()-1] == FromStr::from_str("114438966298574221400558587944897110775439695130591918280658208104532468844382").unwrap(), "polynomial last element wasn't correct");
    
    println!("verifying low degree proof...");
    if !verify_low_degree_proof(&proof.l_merkle_root, &G2, &proof.fri_proof, &BigUint::from(num_steps * 2), &modulus, Some(EXTENSION_FACTOR as u32)) {
        return false;
    }

    println!("performing spot checks");

    // TODO perform spot checks

    return true;
}

fn main() {
    let mut file = File::open("proof.bin").unwrap();

    let proof = StarkProof::deserialize(file).expect("couldn't deserialize"); 
    const LOG_STEPS: usize = 13;
    let mut constants: Vec<BigUint> = Vec::new();
    let modulus: BigUint = BigUint::from_str(MODULUS).expect("modulus couldn't be deserialized into bigint");

    for i in 0..64 {
        let constant = BigUint::from(i as u8).pow(BigUint::from(7u8)) ^ BigUint::from(42u8);
        constants.push(constant);
    }

    if !verify_mimc_proof(BigUint::from(3u8), 2usize.pow(LOG_STEPS as u32), &constants, mimc(&BigUint::from(3u8), 2usize.pow(LOG_STEPS as u32), &constants, &modulus), proof, &modulus) {
        panic!("could not verify mimc stark proof");
    }

}
