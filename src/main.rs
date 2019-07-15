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

use num_bigint::{BigInt, BigUint};
use num_bigint::Sign;
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
use crate::utils::{is_power_of_2, get_pseudorandom_indices, mimc, as_u32_le, multi_interp_4, eval_quartic};
use crate::fft::fft_inv;

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
        //let (root2, column_branches, poly_branches) = p;
        let special_x = BigInt::from_bytes_be(Sign::Plus, root);

        let ys = get_pseudorandom_indices(&m_proof.root2, 40, (rou_deg / 4) as u32, excludeMultiplesOf);

        let column_values = m_proof.column_branches.verify(&ys, None).unwrap();

        // let poly_positions = sum([[y + (roudeg // 4) * j for j in range(4)] for y in &ys], []);
        let mut poly_positions: Vec<u32> = Vec::new();

        for y in &ys {
            for i in 0..4 {
                poly_positions.push(y + ( (rou_deg as u32) / 4) * i); 
            }
        }

        let poly_values = m_proof.poly_branches.verify(&poly_positions, Some(root.clone())).unwrap();

        /*
        For each y coordinate, get the x coordinates on the row, the values on the row, and the value at that y from the column
        */

        /*
        println!("poly values {} are", poly_values.len());

        for val in &poly_values {
            println!("{}", hex::encode(&val));
        }
        */

        let mut xcoords: Vec<BigInt> = Vec::new();
        let mut rows: Vec<BigInt> = Vec::new();

        for (i, y) in (&ys).iter().enumerate() {
            let x1 = root_of_unity.modpow(&BigInt::from(*y), &modulus);

            for j in 0..4 {
                //println!("x1 is {}", &x1);
                //println!("quartic root of unity is {}", &quartic_roots_of_unity[j]);

                xcoords.push(((&quartic_roots_of_unity[j]) * &x1) % modulus);
                rows.push(BigInt::from_bytes_be(Sign::Plus, &poly_values[i*4 + j]));
            }
        }

        /*
        Verify for each selected y coordinate that the four points from the polynomial and the one point from the column that are on that y coordinate are on the same deg < 4 polynomial
        */

        // println!("len xcoords / 4 is {}", xcoords.len() / 4);
        // println!("len rows / 4 is {}", rows.len() / 4);

        let polys: Vec<BigInt> = multi_interp_4(&xcoords, &rows, modulus);

        for (p, c) in polys.chunks(4).zip(column_values.iter()) {
            assert!(eval_quartic(&p, &special_x, modulus) == BigInt::from_bytes_be(Sign::Plus, c), "low degree test failed...");
        }

        root_of_unity = root_of_unity.modpow(&BigInt::from(4u8), &modulus); 
        max_deg_plus_1 = max_deg_plus_1 / BigInt::from(4u8);
        rou_deg = rou_deg / 4;
        root = &m_proof.root2;
    }

    // verify the polynomial

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
    assert!(constants_mini_polynomial[constants_mini_polynomial.len()-1] == FromStr::from_str("114438966298574221400558587944897110775439695130591918280658208104532468844382").unwrap(), "polynomial last element wasn't correct");
    
    println!("verifying low degree proof...");
    if !verify_low_degree_proof(&proof.l_merkle_root, G2.clone(), &proof.fri_proof, BigInt::from(num_steps * 2), &modulus, Some(EXTENSION_FACTOR as u32)) {
        return false;
    }

    println!("low degree proof verified...\nperforming spot checks");

    // TODO perform spot checks

    return true;
}

fn main() {
    let mut file = File::open("proof.bin").unwrap();

    let proof = StarkProof::deserialize(file).expect("couldn't deserialize"); 
    const LOG_STEPS: usize = 13;
    let mut constants: Vec<BigInt> = Vec::new();
    let modulus: BigInt = BigInt::from_str(MODULUS).expect("modulus couldn't be deserialized into bigint");

    for i in 0..64 {
        let constant = BigInt::from(i as u8).pow(BigUint::from(7u8)) ^ BigInt::from(42u8);
        constants.push(constant);
    }

    if !verify_mimc_proof(BigInt::from(3u8), 2usize.pow(LOG_STEPS as u32), &constants, mimc(&BigInt::from(3u8), 2usize.pow(LOG_STEPS as u32), &constants, &modulus), proof, &modulus) {
        panic!("could not verify mimc stark proof");
    }
}
