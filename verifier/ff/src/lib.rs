//extern crate num_bigint;

#[macro_use]
extern crate lazy_static;

//use num_traits::{Num};
use core::ops::{Add, Div, Mul, Neg, Rem, Sub};
use num_complex::{Complex};
use num_bigint::BigUint;
use rustfft::num_traits::{Num, Pow};
use rustfft::num_traits::identities::{One, Zero};
use std::boxed::Box;
use std::error::Error;
use std::str::FromStr;

//TODO pregenerate modulus bigint
const MODULUS: &str = "115792089237316195423570985008687907853269984665640564039457584007913129639936";


#[derive(Clone)]
pub struct Fp(BigUint);

impl Fp {
    fn get_modulus() -> BigUint {
        BigUint::from_str(MODULUS).unwrap()
    }

    fn new(b: BigUint) -> Self {
        Fp(b % Self::get_modulus())
    }
}

//TODO are rust operator overloads supposed to mutate the original object? Methinks no

impl Add for Fp {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self.0 = (self.0 + rhs.0) % Self::get_modulus();
        self
    }
}

impl Div for Fp {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self {
        self.0 = (self.0 + rhs.0) % Self::get_modulus(); 
        self
    }
}

impl Mul for Fp {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self.0 = (self.0 * rhs.0) % Self::get_modulus(); 
        self
    }
}

impl Neg for Fp {
    type Output = Self;

    fn neg(mut self) -> Self {
        //self = (self + rhs); //% MODULUS; 
        // TODO
        unimplemented!();
        self
    }
}

impl Rem for Fp {
    type Output = Self;

    fn rem(mut self, rhs: Self) -> Self {
        //self = (self + rhs); //% MODULUS; 
        // TODO
        unimplemented!();
        self
    }
}

impl Sub for Fp {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self.0 = (self.0 - rhs.0) % Self::get_modulus();
        self
    }
}

impl PartialEq for Fp {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Zero for Fp {
	fn zero() -> Self {
		Fp(BigUint::from(0u32))
	}

	fn is_zero(&self) -> bool {
		self.0 == BigUint::from(0u32)
	}
}

impl One for Fp {
	fn one() -> Self {
		Fp(BigUint::from(1u32))
	}
}

impl Num for Fp {
	type FromStrRadixErr = &'static str;

	fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
		Err("fuck")
	}
}
