use num_bigint::BigInt;
use crate::utils::{negative_to_positive};

// do (a-b)%modulus where a may be greater than b
// inspiration: https://internals.rust-lang.org/t/mathematical-modulo-operator/5952
fn submod(a: &BigInt, b: &BigInt, modulus: &BigInt) -> BigInt {
    match b > a {
        true => {
            let res = b - a;
            negative_to_positive(&res, modulus)
        },
        false => {
            (a-b) % modulus
        }
    }
}

fn simple_ft(vals: &Vec<BigInt>, roots_of_unity: &Vec<BigInt>, modulus: &BigInt) -> Vec<BigInt> {
    if vals.len() > 4 {
        panic!("called ft with more than four arguments");
    }

    let mut output: Vec<BigInt> = Vec::new();

    for i in 0..roots_of_unity.len() {
        let mut last = BigInt::from(0u8);
        for j in 0..roots_of_unity.len() {
            last += vals[j].clone() * &roots_of_unity[(i*j) % roots_of_unity.len()];
        }

        output.push(last % modulus);
    }

    output
}

fn _fft(v: &Vec<BigInt>, roots: &Vec<BigInt>, modulus: &BigInt) -> Vec<BigInt> {
    if v.len() <= 4 {
        return simple_ft(v, roots, &modulus);
    }

    let right_vals: Vec<BigInt> = v.iter().enumerate().filter(|&(i, _)| i % 2 != 0).map(|(_, e)| e.clone()).collect();
    let left_vals: Vec<BigInt>  = v.iter().enumerate().filter(|&(i, _)| i % 2 == 0).map(|(_, e)| e.clone()).collect();
    let new_roots: Vec<BigInt> = roots.iter().enumerate().filter(|&(i, _)| i % 2 == 0).map(|(_, e)| e.clone()).collect();

    let left = _fft(&left_vals, &new_roots, &modulus);
    let right = _fft(&right_vals, &new_roots, &modulus); 

    let mut output: Vec<BigInt> = vec![BigInt::from(0u32); v.len()];

    // TODO why does y not need to be dereferenced here?
    for (i, (x, y)) in left.iter().zip(right).enumerate() {
        let y_times_root: BigInt = y * &roots[i];

        output[i] = negative_to_positive(&(x+&y_times_root.clone()), modulus);

        output[i+left.len()] = negative_to_positive(&(x - &y_times_root), &modulus);
    }

    output
}

// inverse fast fourier transform
pub fn fft_inv(v: &Vec<BigInt>, root_of_unity: &BigInt, modulus: &BigInt) -> Vec<BigInt> {
    let mut roots_of_unity: Vec<BigInt>  = vec![BigInt::from(1u32), root_of_unity.clone()];
    let mut vals = v.clone();
    let one = BigInt::from(1u32);

    while roots_of_unity[roots_of_unity.len()-1] != one {
        let new_root = (roots_of_unity[roots_of_unity.len()-1].clone() * root_of_unity.clone()) % modulus;
        roots_of_unity.push(new_root);
    }

    if roots_of_unity.len() > vals.len() {
        // TODO optimize this so that no array copying is done
        roots_of_unity.append(&mut vec![BigInt::from(0u32); roots_of_unity.len() - vals.len() - 1]);
    }

    roots_of_unity.reverse();
    roots_of_unity.remove(roots_of_unity.len()-1);

    let invlen = BigInt::from(vals.len()).modpow(&(modulus-BigInt::from(2u8)), &modulus);
    let mut result: Vec<BigInt> = _fft(v, &roots_of_unity, modulus);
    result = result.iter().map(|x| (x.clone() * &invlen) % modulus).collect();
    result
}
