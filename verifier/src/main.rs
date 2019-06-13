/*
Proof:
    merkle roots of P, D, spot check merkle proofs, low-degree proofs of P and D
*/

pub mod proof;

use ff::{Fp};
use num_bigint::BigUint;
use rustfft::num_traits::pow;
use blake2::{Blake2b, Digest};
use std::mem::transmute;
use self::proof::{StarkProof, LowDegreeProofElement};

const EXTENSION_FACTOR: u32 = 8;

fn get_pseudorandom_indices(seed: &[u8; 32], count: usize /*TODO excludeMultiplesOf?*/) -> Vec<u32> {
    let mut hasher = Blake2b::default();
    let mut hashes: Vec<u8> = Vec::with_capacity(count as usize / 4 + 1);
    let mut output: Vec<u32> = Vec::with_capacity(count as usize);

    hasher.input(seed);
    hashes[..32].clone_from_slice(&hasher.result().clone()[0..32]);

    for i in 0..(count / 4 + 1) {
        hasher = Blake2b::default();
        hasher.input(&hashes[i..i+4]);
        hashes.extend_from_slice(&hasher.result().clone());
    }

    for j in 0..count {
        let mut index = [0u8; 4];
        index.clone_from_slice(&hashes[j..j+4]);

        unsafe {
            output[j] = transmute::<[u8; 4], u32>(index);
        }
    }

    output 
}

// https://stackoverflow.com/questions/600293/how-to-check-if-a-number-is-a-power-of-2
fn is_power_of_2(n: u32) -> bool {
    if n == 0 {
        false
    } else {
        n & (n-1) == 0
    }
}

fn verify_low_degree_proof(merkle_root: &[u8; 32], root_of_unity: &Fp, proof: Vec<LowDegreeProofElement>, max_deg_plus_1: &Fp, modulus: &BigUint) -> bool {
    let mut test_val = root_of_unity.clone(); 
    //let mut rou_deg = Fp::new(BigUint::from(1u32));
    let mut rou_deg: usize = 1;

    while test_val != Fp::new(BigUint::from(1u32)) {
        rou_deg = rou_deg * 2;
        test_val.set_internal_value(test_val.internal_value().modpow(&test_val.internal_value(), &modulus).clone());
    }

    let quartic_roots_of_unity = [
        Fp::new(BigUint::from(1u32)),
        pow(root_of_unity.clone(), rou_deg / 4),
        pow(root_of_unity.clone(), rou_deg / 2),
        pow(root_of_unity.clone(), rou_deg * 3 / 4)
    ];

    // TODO do I need floor() above?

    for element in proof {
        //let (root2, column_branches, poly_branches) = p;
        let special_x = unsafe { 
            Fp::new(BigUint::from_bytes_be(merkle_root));
        };

        let ys = get_pseudorandom_indices(&element.root2, rou_deg / 4/*TODO excludeMultiplesOF?*/);
    }

    true
}

fn _fft(v: &Vec<u32>, roots: &Vec<Fp>) -> Vec<Fp> {
  /*
    if v.len() < 4 {
        return simple_fft(v, roots)
    }
TODO
    */

    let left_vals: Vec<u32> = v.iter().enumerate().filter(|&(i, _)| (i+1) % 2 == 0).map(|(_, e)| *e).collect();
    let right_vals: Vec<u32>  = v.iter().enumerate().filter(|&(i, _)| i % 2 == 0).map(|(_, e)| *e).collect();
    let new_roots: Vec<Fp> = roots.iter().enumerate().filter(|&(i, _)| (i+1) % 2 == 0).map(|(_, e)| e.clone()).collect();

    let left = _fft(&left_vals, &new_roots);
    let right = _fft(&right_vals, &new_roots); 
    let mut output: Vec<Fp> = vec![Fp::new(BigUint::from(0u32)); v.len()];

    // TODO why does y not need to be dereferenced here?
    for (i, (x, y)) in left.iter().zip(right).enumerate() {
        let y_times_root: BigUint = y.internal_value() * roots[i].internal_value();
        output[i] = Fp::new(x.internal_value()+y_times_root.clone());
        output[i+left.len()] = Fp::new(x.internal_value()-y_times_root);
    }

    output
}

// inverse fast fourier transform
fn fft_inv(v: &Vec<u32>, root_of_unity: &Fp) -> Vec<Fp> {
    let mut roots_of_unity: Vec<Fp>  = vec![Fp::new(1u32.into()), root_of_unity.clone()];
    let mut vals = v.clone();
    //let const modulus = Fp::get_modulus();

    while roots_of_unity[roots_of_unity.len()-1] != Fp::new(BigUint::from(1u32)) {
        roots_of_unity.push(roots_of_unity[roots_of_unity.len()-1].clone() * root_of_unity.clone())
    }

    if roots_of_unity.len() > vals.len() {
        // TODO optimize this so that no array copying is done
        roots_of_unity.append(&mut vec![Fp::new(BigUint::from(0u32)), Fp::new(BigUint::from(roots_of_unity.len() - vals.len() - 1))]);
    }

    //let inv = true;

    //if inv {
        _fft(v, &roots_of_unity)
    //}
}

fn verify_mimc_proof(inp: u32, num_steps: u32, round_constants: &Vec<u32>, output: u32, proof: StarkProof) -> bool {
    let modulus: BigUint = Fp::get_modulus();

    if num_steps > (2u32.pow(32u32) / EXTENSION_FACTOR) { //TODO use of floor here?
        return false;
    }

    if !is_power_of_2(num_steps) || !is_power_of_2(round_constants.len() as u32) {
        return false;
    }

    if (round_constants.len() as u32) < num_steps {
        return false;
    }

    let precision = num_steps * EXTENSION_FACTOR;
    let G2: BigUint = BigUint::from(7u32).modpow(&((modulus.clone() - BigUint::from(1u32)) / precision), &modulus); // TODO do I need floor() here for some reason?
    let skips = precision / num_steps;
    let constants_mini_polynomial = fft_inv(round_constants, &Fp::new(G2.modpow(&BigUint::from(EXTENSION_FACTOR*skips), &modulus)));
    
    if !verify_low_degree_proof(&proof.l_merkle_root, &Fp::new(G2.clone()), proof.fri_proof, &Fp::new(BigUint::from(num_steps * 2)), &modulus /*exclude_multiples_of=extension_factor*/) {
        return false;
    }

    // TODO perform spot checks

    return true;
}

fn main() {

}
