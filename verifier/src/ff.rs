extern crate num_bigint;

//use num_traits::{Num};
use core::ops::{Add, Div, Mul, Neg, Rem, Sub};
use num_complex::{Complex};
use num_bigint::BigUint;
use rustfft::num_traits::Num;
use rustfft::num_traits::identities::{One, Zero};
use std::boxed::Box;


const MODULUS: BigUint = BigUint::pow(BigUint::from(2), BigUint::from(256)) - BigUInt::pow(BigUint::from(2), BigUint::from(32)) * BigUint::from(351) + BigUint::from(1);

#[derive(Clone)]
pub struct Fp(BigUint);

impl Fp {
    fn Foo(bar: Complex<Fp>, baz: Complex<Fp>) {
        return bar + baz;
    }
}

impl Add for Fp {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self = (self + rhs); //% MODULUS; 
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
        self
    }
}

impl Rem for Fp {
    type Output = Self;

    fn rem(mut self, rhs: Self) -> Self {
        self = (self + rhs); //% MODULUS; 
        self
    }
}

impl Sub for Fp {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self = (self + rhs); //% MODULUS; 
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
	type FromStrRadixErr = Error;

	fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
		Err(From::from("fuck"))
	}
}
