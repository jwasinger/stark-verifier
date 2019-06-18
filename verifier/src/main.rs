/*
Proof:
    merkle roots of P, D, spot check merkle proofs, low-degree proofs of P and D
*/

pub mod proof;

use num_bigint::BigUint;
use rustfft::num_traits::Pow;
use std::str::FromStr;

use blake2::{Blake2b, Digest};
use std::mem::transmute;
use self::proof::{StarkProof, LowDegreeProofElement};

const EXTENSION_FACTOR: usize = 8;
const MODULUS: &str = "115792089237316195423570985008687907853269984665640564039457584006405596119041";

fn mimc(input: &BigUint, steps: usize, round_constants: &Vec<BigUint>, modulus: &BigUint) -> BigUint {
    let mut output = input.clone();

	for i in 0..(steps-1) {
      output = output.pow(3u32);
      let mut x = round_constants[i % round_constants.len()].clone() % modulus.clone();
      output = output + x;

      output = round_constants[i % round_constants.len()].clone() % modulus.clone();
	}

    output
}

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

fn verify_low_degree_proof(merkle_root: &[u8; 32], root_of_unity: &BigUint, proof: Vec<LowDegreeProofElement>, max_deg_plus_1: &BigUint, modulus: &BigUint) -> bool {
    let mut test_val = root_of_unity.clone(); 
    //let mut rou_deg = Fp::new(BigUint::from(1u32));
    let mut rou_deg: usize = 1;

    while test_val != BigUint::from(1u32) {
        rou_deg = rou_deg * 2;
        test_val = test_val.modpow(&test_val, &modulus).clone();
    }

    let quartic_roots_of_unity = [
        BigUint::from(1u32),
        root_of_unity.pow(rou_deg / 4),
        root_of_unity.pow(rou_deg / 2),
        root_of_unity.pow(rou_deg * 3 / 4)
    ];

    // TODO do I need floor() above?

    for element in proof {
        //let (root2, column_branches, poly_branches) = p;
        let special_x = BigUint::from_bytes_be(merkle_root);

        let ys = get_pseudorandom_indices(&element.root2, rou_deg / 4/*TODO excludeMultiplesOF?*/);
    }

    true
}

fn simple_ft(vals: &Vec<BigUint>, roots_of_unity: &Vec<BigUint>, modulus: &BigUint) -> Vec<BigUint> {
    if vals.len() > 4 {
        panic!("called ft with more than four arguments");
    }

    let mut output: Vec<BigUint> = Vec::new();

    /*
    println!("simple ft vals");
    for val in vals {
        println!("{}", &val);
    }

    println!("simple ft roots");
    for root in roots_of_unity {
        println!("{}", &root);
    }
    */

    for i in 0..roots_of_unity.len() {
        let mut last = BigUint::from(0u8);
        for j in 0..roots_of_unity.len() {
            last += vals[j].clone() * &roots_of_unity[(i*j) % roots_of_unity.len()];
        }

        output.push(last % modulus);
    }

    output
}

// do (a-b)%modulus where a may be greater than b
// inspiration: https://internals.rust-lang.org/t/mathematical-modulo-operator/5952
fn _magic(a: &BigUint, b: &BigUint, modulus: &BigUint) -> BigUint {
    match b > a {
        true => {
            let res = b - a;
            // want to find a number modulus * k + res > 0 
            let mut k: BigUint = BigUint::from(1u8);
            let mul_fac: BigUint = BigUint::from(10u8);

            loop {
              // println!("k is {}", modulus * &k);
              //println!("res is {}", res);
              if (modulus * &k) > res {
                    let res = ((modulus * &k) - res) % modulus;
                    // println!("modulus * k - res % modulus = {}", &res);
                    return res;
              }

              k = k * &mul_fac;
            }
        },
        false => {
            return (a-b) % modulus;
        }
    }
}

fn _fft(v: &Vec<BigUint>, roots: &Vec<BigUint>, modulus: &BigUint) -> Vec<BigUint> {
    if v.len() <= 4 {
        return simple_ft(v, roots, &modulus);
    }

    let right_vals: Vec<BigUint> = v.iter().enumerate().filter(|&(i, _)| i % 2 != 0).map(|(_, e)| e.clone()).collect();
    let left_vals: Vec<BigUint>  = v.iter().enumerate().filter(|&(i, _)| i % 2 == 0).map(|(_, e)| e.clone()).collect();
    let new_roots: Vec<BigUint> = roots.iter().enumerate().filter(|&(i, _)| i % 2 == 0).map(|(_, e)| e.clone()).collect();

    let left = _fft(&left_vals, &new_roots, &modulus);
    let right = _fft(&right_vals, &new_roots, &modulus); 

    /*
    println!("left is: ");
    for val in &left {
        println!("{}", &val);
    }

    println!("right is: ");
    for val in &right {
        println!("{}", &val);
    }
    */

    let mut output: Vec<BigUint> = vec![BigUint::from(0u32); v.len()];

    // TODO why does y not need to be dereferenced here?
    for (i, (x, y)) in left.iter().zip(right).enumerate() {
        let y_times_root: BigUint = y * &roots[i];

        output[i] = x+&y_times_root.clone() % modulus;
        //println!("x {}, y {}, z {}, a {}", x, x-&y_times_root, (x-&y_times_root) % modulus);

        output[i+left.len()] = _magic(x, &y_times_root, &modulus);

        //println!("(x-y_times_root) % modulus = {}", output[i+left.len()]
        /*
        output[i+left.len()] = match x >= &y_times_root {
            true => (x-&y_times_root) % modulus,
            false => (&y_times_root - x) % modulus
        };
        */
        
        //println!("y times root = {}", &y_times_root);
        //println!("modulus = {}", &modulus);

        /*
        if x >= &y_times_root {
            println!("x-y_times_root = {}", x-&y_times_root);
            println!("(x-y_times_root) % modulus = {}", (x-&y_times_root) % modulus);
        } else {
            println!("y_times_root - x = {}", &y_times_root - x);
            println!("(y_times_root-x) % modulus = {}", (&y_times_root - x) % modulus);
        }
        */

        //println!("O[i] = {}", output[i]);
        //println!("O[i+len(L)] = {}", output[i+left.len()]);
    }

    /*
    println!("output is: ");
    for ref item in &output {
        println!("{}", item);
    }
    */


    output
}

// inverse fast fourier transform
fn fft_inv(v: &Vec<BigUint>, root_of_unity: &BigUint, modulus: &BigUint) -> Vec<BigUint> {
    let mut roots_of_unity: Vec<BigUint>  = vec![BigUint::from(1u32), root_of_unity.clone()];
    let mut vals = v.clone();

    //let const modulus = Fp::get_modulus();

   // println!("root of unity is {}", &root_of_unity);
    let one = BigUint::from(1u32);
    while roots_of_unity[roots_of_unity.len()-1] != one {
        let new_root = (roots_of_unity[roots_of_unity.len()-1].clone() * root_of_unity.clone()) % modulus;
        roots_of_unity.push(new_root);
    }

    if roots_of_unity.len() > vals.len() {
        // TODO optimize this so that no array copying is done
        roots_of_unity.append(&mut vec![BigUint::from(0u32); roots_of_unity.len() - vals.len() - 1]);
    }

    roots_of_unity.reverse();
    roots_of_unity.remove(roots_of_unity.len()-1);

    let invlen = BigUint::from(vals.len()).modpow(&(modulus-BigUint::from(2u8)), &modulus);

    /*
    println!("roots of unity: ");
    for root in &roots_of_unity {
        println!("{}", root);
    }
    */

    let mut result: Vec<BigUint> = _fft(v, &roots_of_unity, modulus);
    
    // println!("invlen is {}", &invlen);
    result = result.iter().map(|x| (x.clone() * &invlen) % modulus).collect();
    println!("final result is ");
    for r in &result {
        println!("{}", r);
    }

    //assert!(&result[result.len()-1] == &FromStr::from_str("29192221157829857950777572926076894872131454422527235476297526286525450540865").unwrap(), "unexpected end of output");

    result
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

    //println!("skips is {}", skips);
    //println!("extension factor is {}", EXTENSION_FACTOR);
    //println!("num_steps is {}", num_steps);
    assert!(num_steps == 8192, "num steps incorrect");
    //println!("G2 is {}", G2);
    assert!(G2 == FromStr::from_str("41913712888260089065520476180880993127517355946012995597287997778376518235852").unwrap(), "G2 isn't correct");

    //println!("constants mini polynomial root is {}",  &val);
    assert!(val == FromStr::from_str("56670364103764250102176604807203318908867195832872336813161821519223575486477").unwrap(), "constants mini polynomial root wasn't correct");

    //println!("m-p {}", &((modulus.clone() - BigUint::from(1u32)) / precision));

    /*
    println!("precision {}", precision);
    println!("G2 {}", G2);
    println!("skips {}", skips);
    println!("constants: ");
    */

    /*
    for constant in round_constants {
        println!("{}", constant);
    }
    */

    //println!("val {}", &val.internal_value());
    //println!("modulus {}", &modulus);

    let constants_mini_polynomial = fft_inv(round_constants, &val, &modulus);
    
    /*
    if !verify_low_degree_proof(&proof.l_merkle_root, &Fp::new(G2.clone()), proof.fri_proof, &Fp::new(BigUint::from(num_steps * 2)), &modulus /*exclude_multiples_of=extension_factor*/) {
        return false;
    }
    */

    // TODO perform spot checks

    return true;
}

fn main() {
    let serialized_proof = hex::decode("53a0e380573d3bada3a837e48d93aafa7ef8598cad5164919bbf890f445f04ecf0aeca09558102275bbf9cc82f49d71b37ae96fef1aa3141ae805deb0e80fd14").unwrap();
    let proof = StarkProof::deserialize(&serialized_proof).expect("couldn't deserialize"); 
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
