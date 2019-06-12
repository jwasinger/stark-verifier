/*
Proof:
    merkle roots of P, D, spot check merkle proofs, low-degree proofs of P and D
*/

use ff::{Fp};
use num_bigint::BigUint;
use rustfft::num_traits::Pow;


const EXTENSION_FACTOR: u32 = 8;

// https://stackoverflow.com/questions/600293/how-to-check-if-a-number-is-a-power-of-2
fn is_power_of_2(n: u32) -> bool {
    if n == 0 {
        false
    } else {
        n & (n-1) == 0
    }
}

/*
fn verify_low_degree_proof(merkle_root: &[u8, 32], root_of_unity: &Fp, proof: ..., max_deg_plus_1: &Fp, modulus: &Fp) -> bool {
    let mut test_val = root_of_unity.clone(); 
    let mut rou_deg: f32 = 1

    while test_val != 1 {
        rou_deg *= 2;
        test_val = test_val * test_val;
    }

    let quartic_roots_of_unity = [
        Fp::new(1),
        root_of_unity.pow((roudeg / 4)),
        root_of_unity.pow((roudeg / 2)),
        root_of_unity.pow((roudeg * 3 / 4))
    ];

    // TODO do I need floor() above?

    for p in proof.proof {
        let root2, column_branches, poly_branches = p;
        let special_x = Fp::new(merkle_root)

    }
}
*/
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

fn verify_mimc_proof(inp: u32, num_steps: u32, round_constants: &Vec<u32>, output: u32/*, proof: Proof*/) -> bool {
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
    let G2: BigUint = BigUint::from(7u32).pow((modulus - BigUint::from(1u32)) / precision); // TODO do I need floor() here for some reason?
    let skips = precision / num_steps;
    let constants_mini_polynomial = fft_inv(round_constants, &Fp::new(G2.pow(EXTENSION_FACTOR*skips)));
    
    /*
    //TODO
    if !verify_low_degree_proof(l_root, G2, fri_proof, num_steps * 2, modulus, exclude_multiples_of=extension_factor) {
        return false;
    }
    */

    // TODO perform spot checks

    return true;
}

fn main() {

}
