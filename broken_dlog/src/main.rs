use std::{cmp::min, str::FromStr};
use std::fs;
use num_bigint::BigUint;
use std::collections::HashMap;
use num_traits::{FromPrimitive, One, Zero};
use num_iter::range_step;

fn bsgs(g: BigUint, y: BigUint, p: BigUint, max_power: BigUint) -> Option<BigUint> {
    let m = max_power.sqrt() + 1u32;

    let mut table = HashMap::new();
    let mut g_raised_to_j = BigUint::one();
    for j in range_step(BigUint::zero(), m.clone(), BigUint::one()) {
        table.insert(g_raised_to_j.clone(), j.clone());
        g_raised_to_j = (&g_raised_to_j * &g) % &p;
    }

    let g_raised_to_minus_m = mod_exp(&g, &(&p - &m - BigUint::one()), &p);
    let mut temp = y;
    for i in range_step(BigUint::zero(), m.clone(), BigUint::one()) {
        if let Some(j) = table.get(&temp) {
            let potential_x = i * &m + j;
            if potential_x < max_power {
                return Some(potential_x);
            }
        }
        temp = (temp * &g_raised_to_minus_m) % &p;
    }

    None
}

fn mod_exp(base: &BigUint, exp: &BigUint, modulus: &BigUint) -> BigUint {
    let mut result = BigUint::one();
    let mut base = base.clone() % modulus;
    let mut exp = exp.clone();

    while !exp.is_zero() {
        if &exp % 2u32 == BigUint::one() {
            result = (result * &base) % modulus;
        }
        exp >>= 1;
        base = (&base * &base) % modulus;
    }

    result
}

fn main() {
    let filename = "input.txt";
    let contents = fs::read_to_string(filename).expect("Error reading file");

    let mut g_str = String::new();
    let mut h_str = String::new();
    let mut a_str = String::new();
    let mut p_str = String::new();
    let mut r_str = String::new();
    let mut c_str = String::new();
    let mut t_str = String::new();
    let mut r_exists = false;

    for line in contents.lines() {
        if line.starts_with("g=") {
            g_str = line.chars().skip(2).collect();
        } else if line.starts_with("h=") {
            h_str = line.chars().skip(2).collect();
        } else if line.starts_with("a=") {
            a_str = line.chars().skip(2).collect();
        } else if line.starts_with("p=") {
            p_str = line.chars().skip(2).collect();
        } else if line.starts_with("c=") {
            c_str = line.chars().skip(2).collect();
        } else if line.starts_with("t=") {
            t_str = line.chars().skip(2).collect();
        } else if line.starts_with("r=") {
            r_str = line.chars().skip(2).collect();
            r_exists = true;
        }
    }

    let p = BigUint::from_str(&p_str).unwrap();
    let g = BigUint::from_str(&g_str).unwrap();
    let h = BigUint::from_str(&h_str).unwrap();
    let a = BigUint::from_str(&a_str).unwrap();
    let mut r = BigUint::from_str(&r_str).unwrap();
    let c = BigUint::from_str(&c_str).unwrap();
    let t = BigUint::from_str(&t_str).unwrap();

    assert_eq!(g.modpow(&t, &p), (&a * h.modpow(&c, &p)) % &p);

    if !r_exists {
        let max_power = min(BigUint::from_u128(2u128.pow(50)).unwrap(), p.clone());
        match bsgs(g.clone(), h.clone(), p.clone(), max_power) {
            Some(x) => {
                println!("r is {}", x);
                r = x;
            },
            None => println!("No solution found"),
        }
    }

    assert_eq!(g.modpow(&r, &p), a);

    let x = ((t - r) % &p / c) % &p;
    let derived_h = g.modpow(&x, &p);
    assert_eq!(derived_h, h);

    println!("Secret Knowledge X: {}", x);
}
