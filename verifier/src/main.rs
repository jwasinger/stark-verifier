/*
Proof:
    merkle roots of P, D, spot check merkle proofs, low-degree proofs of P and D
*/

use ff::{Fp};


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
    let mut rou_deg = 1

    while test_val != 1 {
        rou_deg *= 2;
        test_val = test_val * test_val;
    }

    let quartic_roots_of_unity = [
        Fp(1),
        root_of_unity.pow(floor(roudeg / 4)),
        root_of_unity.pow(floor(roudeg / 2)),
        root_of_unity.pow(floor(roudeg * 3 / 4))
    ];

    for p in proof.proof {
        let root2, column_branches, poly_branches = p;
        let special_x = Fp::new(merkle_root)

    }
}
*/
fn _fft(v: &Vec<Fp>, roots: &Vec<Fp>) -> Vec<Fp> {
  /*
    if v.len() < 4 {
        return simple_fft(v, roots)
    }
TODO
    */

    let left_vals = v.iter().enumerate().filter(|&(i, _)| (i+1) % 2 == 0).map(|(_, e)| e);
    let right_vals = v.iter().enumerate().filter(|&(i, _)| i % 2 == 0).map(|(_, e)| e);
    let new_roots = roots.iter().enumerate().filter(|&(i, _)| (i+1) % 2 == 0).map(|(_, e)| e);

    let left = _fft(&left_vals, new_roots);
    let right = _fft(&right_vals, new_roots); 
    let output: Vec<Fp> = vec![0; v.len()];

    for (i, (x, y)) in left.iter().zip(right).enumerate() {
        let y_times_root = y * roots[i];
        output[i] = x+y_times_root;
        output[i+left.len()] = x-y_times_root;
    }

    output
}

fn fft_inv(v: &Vec<Fp>, root_of_unity: &Fp) -> Vec<Fp> {
    let mut roots_of_unity: Vec<Fp>  = vec![Fp::new(1.into()), root_of_unity];
    let mut vals = v.clone();
    //let const modulus = Fp::get_modulus();

    while roots_of_unity[roots_of_unity.len()-1] != 1 {
        roots_of_unity.push(roots_of_unity[roots_of_unity.len()-1] * root_of_unity)
    }

    if roots_of_unity.len() > vals.len() {
        // TODO optimize this so that no array copying is done
        roots_of_unity.append([0, roots_of_unity.len() - vals.len() - 1]);
    }

    let inv = true;

    if inv {
        _fft(&v, roots_of_unity)
    }
}

fn verify_mimc_proof(inp: u32, num_steps: u32, round_constants: &[u32], output: u32/*, proof: Proof*/) -> bool {
    const modulus: Fp = Fp::get_modulus();

    if num_steps > floor(2f32.pow(32f32) / EXTENSION_FACTOR) {
        false
    }

    if !is_a_power_of_2(num_steps) || !is_a_power_of_2(round_constants.len()) {
        false
    }

    if round_constants.len() < num_steps {
        false
    }

    let precision = num_steps * EXTENSION_FACTOR;
    let G2 = Fp(7).pow(floor((modulus - 1) / precision ));
    let skips = floor(precision / num_steps);
    let constants_mini_polynomial = fft_inv(round_constants, G2.pow(EXTENSION_FACTOR*skips2));
    
    /*
    //TODO
    if !verify_low_degree_proof(l_root, G2, fri_proof, num_steps * 2, modulus, exclude_multiples_of=extension_factor) {
        false
    }
    */

    // TODO perform spot checks

    true
}

fn main() {

}
