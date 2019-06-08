/*
Proof:
    merkle roots of P, D, spot check merkle proofs, low-degree proofs of P and D
*/

mod ff;

use std::sync::Arc;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

use polynomial::Polynomial;
// pub use ::ff;
use ff::{Fp};

fn main() {
    let mut input:  Vec<Complex<f32>> = vec![Complex::zero(); 1234];
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); 1234];

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(1234);
    fft.process(&mut input, &mut output);
     
    // The fft instance returned by the planner is stored behind an `Arc`, so it's cheap to clone
    let fft_clone = Arc::clone(&fft);

    let poly = Polynomial::new(vec![1, 2, 3]);
    assert_eq!("1+2*x+3*x^2", poly.pretty("x"));
}


/*
const EXTENSION_FACTOR: u32 = 8;

fn verify_mimc_proof(inp: u32, num_steps: u32, round_constants: &[u32], output: u32, proof: Proof) -> bool {
    if num_steps > floor(pow(2, 32) / EXTENSION_FACTOR) {
        false
    }

    if !is_a_power_of_2(steps) || !is_a_power_of_2(round_constants.len()) {
        false
    }

    if round_constants.len() < num_steps {
        false
    }

    true
}

fn main() {

}
*/
