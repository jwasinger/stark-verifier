use num_bigint::{BigInt, Sign, ToBigInt};
use blake2::{Blake2s, Digest};
use std::mem::transmute;
use rustfft::num_traits::Pow;
use rustfft::num_traits::identities::{Zero, One};
use rustfft::num_traits::sign::Signed;

pub fn mimc(input: &BigInt, steps: usize, round_constants: &Vec<BigInt>, modulus: &BigInt) -> BigInt {
    let mut output = input.clone();

	for i in 0..(steps-1) {
      output = negative_to_positive(&(output.pow(3u32) + round_constants[i % round_constants.len()].clone()), modulus);
	}

    output
}

/* 
  TODO make the first argument not be passed by reference
  Convert a negative number to positive.
  Inspiration:
  https://internals.rust-lang.org/t/mathematical-modulo-operator/5952
  https://math.stackexchange.com/questions/519845/modulo-of-a-negative-number
*/
pub fn negative_to_positive(n: &BigInt, modulus: &BigInt) -> BigInt {
    if !n.is_negative() {
        return n.clone() % modulus;
    }

    let mut k: BigInt = BigInt::from(1u8);
    let mul_fac: BigInt = BigInt::from(10u8);
    let mut res = n.clone() * -BigInt::one();

    loop {
        if (modulus * &k) > res {
            let res = ((modulus * &k) - res) % modulus;
            return res;
        }

        k = k * &mul_fac;
    }
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
    let mut output: Vec<u32> = Vec::new();

    let real_modulus: u32 = match excludeMultiplesOf {
        Some(exclude) => {
            modulus * ( exclude - 1 ) / exclude
        },
        None => {
            modulus
        }
    };

    hashes[0..32].clone_from_slice(seed/*&hasher.result().clone()*/);

    while hashes.len() < 4 * count {
        hasher = Blake2s::default();
        hasher.input(&hashes[hashes.len()-32..]);
        let result = hasher.result();
        hashes.extend_from_slice(&result);
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

pub fn to_u8_array_le(num: u32) -> [u8; 4] {
    [
      (num >> 0) as u8,
      (num >> 8) as u8,
      (num >> 16) as u8,
      (num >> 24) as u8
    ]
}

pub fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}

pub fn eval_quartic(eq: &[BigInt], y: &BigInt, m: &BigInt) -> BigInt {
    assert!(eq.len() == 4, "only quartic equations supported");
    let modulus = &BigInt::from(m.clone());
    let x = BigInt::from(y.clone());
    let xsq = ( &x * &x ) % modulus;
    let xcb =  &x * &xsq;

    let res: BigInt = &(eq[0]) + &(eq[1]) * &x + &(eq[2]) * &xsq + &(eq[3]) * &xcb;

    match res.is_negative() {
        true => {
            negative_to_positive(&res, m) % m 
        },
        false => {
            res % m 
        }
    }
}

pub fn divmod(x: &BigInt, y: &BigInt, m: &BigInt) -> BigInt {
    return (x * &inv(y.clone(), m.clone())) % m;
}

pub fn eval_poly_at(poly: &Vec<BigInt>, x: &BigInt, m: &BigInt) -> BigInt {
    let mut y = BigInt::zero();
    let mut power_of_x = BigInt::one();

    for p_coef in poly.iter() {
        y += &power_of_x * p_coef;
        power_of_x = (power_of_x * x) % m;
    }

    return y % m;
}

// modular inverse
fn inv(x: BigInt, m: BigInt) -> BigInt {
	let a = BigInt::from(x);

    let modulus = BigInt::from(m); 

    if a == BigInt::zero() {
        return BigInt::zero();
    }

    let mut lm = BigInt::one();
    let mut hm = BigInt::zero();
    let mut low = negative_to_positive(&a, &modulus);
    let mut high = modulus.clone();

    while low > BigInt::one() {
        let r = &high / &low;

        let nm = &hm - &lm * &r;
        let new = &high - &low * &r;

        // TODO how can we avoid cloning here (maybe using mem::swap?)
        hm = lm.clone();
		lm = nm;
        high = low.clone();
        low = new;
    }

    negative_to_positive(&lm, &modulus) % &modulus
}

pub fn multi_inv(values: &[BigInt], modulus: &BigInt) -> Vec<BigInt> {
    let mut partials: Vec<BigInt> = vec![BigInt::from(1u8)];

    for i in 0..(values.len()) {
        if values[i] == BigInt::from(0u8) {
            partials.push(&(partials[partials.len()-1]) % modulus);
        } else {
            partials.push(&(partials[partials.len()-1]) * &(values[i]) % modulus);
        }
    }

    let mut inv = inv(partials[partials.len()-1].clone(), modulus.clone());
    let mut outputs: Vec<BigInt> = vec![BigInt::from(0u8); values.len()];

    for i in (1..values.len()+1).rev() {
        if values[i-1] == BigInt::from(0u8) {
            outputs[i-1] = BigInt::from(0u8);
            inv = inv % modulus;
        } else {
            outputs[i-1] = (&(partials[i-1]) * &inv) % modulus;
            inv = (inv * &(values[i-1])) % modulus;
        }
    }

    outputs
}

pub fn multi_interp_4(xsets: &Vec<BigInt>, ysets: &Vec<BigInt>, modulus: &BigInt) -> Vec<BigInt> {
	assert!(xsets.len() == ysets.len(), "number of xs should be equal to number of ys");

    let mut data: Vec<(&[BigInt], [[BigInt; 4]; 4])> = Vec::new();
    let mut inv_targets: Vec<BigInt> = Vec::new();
    let mut output: Vec<BigInt> = Vec::new();

	for i in (0..xsets.len()).step_by(4) {
        let x01 = &(xsets[i]) * &(xsets[i+1]);
        let x02 = &(xsets[i]) * &(xsets[i+2]);
        let x03 = &(xsets[i]) * &(xsets[i+3]);
        let x12 = &(xsets[i+1]) * &(xsets[i+2]);
        let x13 = &(xsets[i+1]) * &(xsets[i+3]);
        let x23 = &(xsets[i+2]) * &(xsets[i+3]);

        let eq0: [BigInt; 4] = [negative_to_positive(&-(&x12 * &(xsets[i+3])), modulus) % modulus, &x12 + &x13 + &x23, -&xsets[i+1] - &(xsets[i+2]) - &(xsets[i+3]), BigInt::from(1u8)];
        let eq1 = [negative_to_positive(&(-&x02 * &(xsets[i+3])), modulus) % modulus, &x02 + &x03 + &x23, -&(xsets[i]) - &(xsets[i+2]) - &(xsets[i+3]), BigInt::from(1u8)];
        let eq2 = [negative_to_positive(&(-&x01 * &(xsets[i+3])), modulus) % modulus, &x01 + &x03 + &x13, -&(xsets[i]) - &(xsets[i+1]) - &(xsets[i+3]), BigInt::from(1u8)];
        let eq3 = [negative_to_positive(&(-&x01 * &(xsets[i+2])), modulus) % modulus, &x01 + &x02 + &x12, -&(xsets[i]) - &(xsets[i+1]) - &(xsets[i+2]), BigInt::from(1u8)];

        let e0 = eval_quartic(&eq0, &xsets[i], modulus);
        let e1 = eval_quartic(&eq1, &xsets[i+1], modulus);
        let e2 = eval_quartic(&eq2, &xsets[i+2], modulus);
        let e3 = eval_quartic(&eq3, &xsets[i+3], modulus);

        data.push((&ysets[i..i+4], [eq0, eq1, eq2, eq3]));
        inv_targets.push(e0);
        inv_targets.push(e1);
        inv_targets.push(e2);
        inv_targets.push(e3);
	}

    let inv_vals = multi_inv(&inv_targets, &modulus);

    for (i, (ys, eqs)) in data.iter().enumerate() {
        let inv_y0 = &(ys[0]) * &(inv_vals[i*4]) % modulus;
        let inv_y1 = &(ys[1]) * &(inv_vals[i*4 + 1]) % modulus;
        let inv_y2 = &(ys[2]) * &(inv_vals[i*4 + 2]) % modulus;
        let inv_y3 = &(ys[3]) * &(inv_vals[i*4 + 3]) % modulus;

        for j in 0..4 {
            let mut output_val = &(eqs[0][j]) * &inv_y0 + &(eqs[1][j]) * &inv_y1 + &(eqs[2][j]) * &inv_y2 + &(eqs[3][j]) * &inv_y3;
            output_val = negative_to_positive(&output_val, modulus) % modulus;
            output.push(output_val);
        }
    }

    output
}

pub fn mul_polys(p0: &Vec<BigInt>, p1: &Vec<BigInt>, modulus: &BigInt) -> Vec<BigInt> {
    assert!(p0.len() == p1.len(), "polynomials must be the same degree");

    //TODO this seems like pointless double memory allocation (but necessary for rust to compile)
    let mut output: Vec<BigInt> = vec![BigInt::zero(); p0.len() + p1.len() - 1];

    for (i, _) in p0.iter().enumerate() {
        for (j, _) in p1.iter().enumerate() {
            output[i+j] += &p0[i] * &p1[j];
        }
    }

    output.iter().map(|x| x  % modulus).collect()
}

pub fn lagrange_interp_2(xs: &[BigInt; 2], ys: &[BigInt; 2], modulus: &BigInt) -> [BigInt; 2] {
    let eq0 = [negative_to_positive(&-&xs[1], modulus), BigInt::one()];
    let eq1 = [negative_to_positive(&-&xs[0], modulus), BigInt::one()];
    let e0 = eval_poly_at(&eq0.to_vec(), &xs[0], modulus);
    let e1 = eval_poly_at(&eq1.to_vec(), &xs[1], modulus);
    let inv_val = inv(&e0 * &e1, modulus.clone());
    let inv_y0 = &ys[0] * &inv_val * &e1;
    let inv_y1 = &ys[1] * &inv_val * &e0;
    let int1 = negative_to_positive(&(&eq0[0] * &inv_y0 + &eq1[0] * &inv_y1), modulus);
    let int2 = negative_to_positive(&(&eq0[1] * &inv_y0 + &eq1[1] * &inv_y1), modulus);

    [int1,
     int2]
}
