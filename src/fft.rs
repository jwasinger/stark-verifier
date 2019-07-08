use num_bigint::BigUint;

// do (a-b)%modulus where a may be greater than b
// inspiration: https://internals.rust-lang.org/t/mathematical-modulo-operator/5952
fn submod(a: &BigUint, b: &BigUint, modulus: &BigUint) -> BigUint {
    match b > a {
        true => {
            let res = b - a;
            // want to find a number modulus * k + res > 0 
            let mut k: BigUint = BigUint::from(1u8);
            let mul_fac: BigUint = BigUint::from(10u8);

            loop {
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

        output[i+left.len()] = submod(x, &y_times_root, &modulus);

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
pub fn fft_inv(v: &Vec<BigUint>, root_of_unity: &BigUint, modulus: &BigUint) -> Vec<BigUint> {
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

    //assert!(&result[result.len()-1] == &FromStr::from_str("29192221157829857950777572926076894872131454422527235476297526286525450540865").unwrap(), "unexpected end of output");

    result
}
