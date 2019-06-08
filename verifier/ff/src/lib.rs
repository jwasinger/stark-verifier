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


lazy_static! {
    static ref MODULUS: BigUint = {
        BigUint::from_str("115792089237316195423570985008687907853269984665640564039457584007913129639936").unwrap()
    };
}

#[derive(Clone)]
pub struct Fp(BigUint);

impl Add for Fp {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self.0 = (self.0 + rhs.0) % MODULUS; 
        self
    }
}

impl Div for Fp {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self {
        self = (self + rhs); //% MODULUS; 
        self
    }
}

impl Mul for Fp {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self = (self + rhs); //% MODULUS; 
        self
    }
}

impl Neg for Fp {
    type Output = Self;

    fn neg(mut self) -> Self {
        //self = (self + rhs); //% MODULUS; 
        unimplemented!();
        self
    }
}

impl Rem for Fp {
    type Output = Self;

    fn rem(mut self, rhs: Self) -> Self {
        //self = (self + rhs); //% MODULUS; 
        unimplemented!();
        self
    }
}

impl Sub for Fp {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self.0 = (self.0 - rhs.0);// % MODULUS;
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
