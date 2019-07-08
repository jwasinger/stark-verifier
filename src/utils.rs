use num_bigint::BigUint;
use blake2::{Blake2s, Digest};
use std::mem::transmute;
use rustfft::num_traits::Pow;

pub fn mimc(input: &BigUint, steps: usize, round_constants: &Vec<BigUint>, modulus: &BigUint) -> BigUint {
    let mut output = input.clone();

	for i in 0..(steps-1) {
      output = output.pow(3u32);
      let mut x = round_constants[i % round_constants.len()].clone() % modulus.clone();
      output = output + x;

      output = round_constants[i % round_constants.len()].clone() % modulus.clone();
	}

    output
}

// https://stackoverflow.com/questions/600293/how-to-check-if-a-number-is-a-power-of-2
pub fn is_power_of_2(n: u32) -> bool {
    if n == 0 {
        false
    } else {
        n & (n-1) == 0
    }
}

pub fn get_pseudorandom_indices(seed: &[u8; 32], count: usize, modulus: u32, excludeMultiplesOf: Option<u32>) -> Vec<u32> {
    let mut hasher = Blake2s::default();
    let mut hashes: Vec<u8> = vec![0u8; 32];
    let mut output: Vec<u32> = Vec::new(); //vec![0u32; count as usize];

    let real_modulus: u32 = match excludeMultiplesOf {
        Some(exclude) => {
            modulus * ( exclude - 1 ) / exclude
        },
        None => {
            modulus
        }
    };

    //println!("modulus is {}", modulus);
    //println!("start seed is {:x?}", seed);
    hashes[0..32].clone_from_slice(seed/*&hasher.result().clone()*/);
    //println!("hashes at start is {:x?}", &hashes);

    //println!("doing for count {}", count);
    while hashes.len() < 4 * count {
        hasher = Blake2s::default();
        hasher.input(&hashes[hashes.len()-32..]);
        let result = hasher.result();
        //println!("input is {:x?}", &hashes[hashes.len()-32..]);
        //println!("output is {:x?}", &result);
        hashes.extend_from_slice(&result);
        //println!("size is {}", hashes.len());
    }

    for j in (0..(count*4)).step_by(4) {
        let mut index = [0u8; 4];
        index.clone_from_slice(&hashes[j..j+4]);
        index = [index[3], index[2], index[1], index[0]];

        unsafe {
            let x = transmute::<[u8; 4], u32>(index);
            //println!("x is {:x?}", &x);
            //println!("x mod modulus is {:x?}", x % &real_modulus);
            output.push(transmute::<[u8; 4], u32>(index) % real_modulus);
        }
    }

    if let Some(exclude) = excludeMultiplesOf {
        output = output.iter().map(|x| 1+x+(x/(exclude-1))).collect();
    }

    output 
}

pub fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}
