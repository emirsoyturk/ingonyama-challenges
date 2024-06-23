use lambdaworks_math::{
    cyclic_group::IsGroup, elliptic_curve::short_weierstrass::{
            curves::bls12_381::{
                curve::BLS12381Curve,
                default_types::FrField, field_extension::BLS12381PrimeField
            },
            point::ShortWeierstrassProjectivePoint,
        }, fft::cpu::roots_of_unity, field::{element::FieldElement, traits::RootsConfig}, polynomial::Polynomial
};
use rand::RngCore;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub type Fr = FieldElement<FrField>;
pub type Fq = FieldElement<BLS12381PrimeField>;

pub fn random_poly(degree: usize) -> Polynomial<Fr> {
    let mut rnd = rand::thread_rng();
    let coefficients: Vec<Fr> = (0..degree).map(|_| Fr::from(rnd.next_u64())).collect();
    Polynomial::new(&coefficients)
}

pub fn fft_over_ec(a: &[Fr], generator: ShortWeierstrassProjectivePoint<BLS12381Curve>) -> Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>> {
    let lagrange = Polynomial::interpolate_fft::<FrField>(a).unwrap();
    lagrange.coefficients.into_iter().map(|coeff| generator.operate_with_self(coeff.representative())).collect()
}

pub fn monomial_to_lagrange_pre_computation(a: &[ShortWeierstrassProjectivePoint<BLS12381Curve>], degree: u64) -> Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>> {
    let roots_of_unity: Vec<Fr> = roots_of_unity::get_powers_of_primitive_root(32, degree as usize, RootsConfig::Natural).unwrap();
    let degree_usize = degree as usize;

    let pairs: Vec<(usize, usize)> = (0..degree_usize)
        .flat_map(|i| (0..degree_usize).filter(move |&j| i != j).map(move |j| (i, j)))
        .collect();

    let inverses: Vec<Fr> = pairs.into_par_iter().map(|(i, j)| {
        let den = &roots_of_unity[i] - &roots_of_unity[j];
        den.inv().unwrap()
    }).collect();


    let mut srs_lagrange = vec![ShortWeierstrassProjectivePoint::neutral_element(); degree_usize];

    srs_lagrange.par_iter_mut().enumerate().for_each(|(i, lagrange_poly)| {
        let root = &roots_of_unity[i];
        for (j, other_root) in roots_of_unity.iter().enumerate() {
            if i != j {
                let num = root - other_root;
                let term = &num * &inverses[j];
                *lagrange_poly = lagrange_poly.operate_with(&a[j].operate_with_self(term.representative()));
            }
        }
    });

    srs_lagrange
}

pub fn monomial_to_lagrange(a: &[ShortWeierstrassProjectivePoint<BLS12381Curve>], degree: u64) -> Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>> {
    let roots_of_unity: Vec<Fr> = roots_of_unity::get_powers_of_primitive_root(32, degree as usize, RootsConfig::Natural).unwrap();
    let degree_usize = degree as usize;

    let mut srs_lagrange = vec![ShortWeierstrassProjectivePoint::neutral_element(); degree_usize];
    
    let start_time = std::time::Instant::now();

    srs_lagrange.par_iter_mut().enumerate().for_each(|(i, lagrange_poly)| {
        let root = &roots_of_unity[i];
        for (j, other_root) in roots_of_unity.iter().enumerate() {
            if i != j {
                let num = root - other_root;
                let term = &num * &num.inv().unwrap();
                *lagrange_poly = lagrange_poly.operate_with(&a[j].operate_with_self(term.representative()));
            }
        }
    });
    
    println!("Time to compute lagrange: {:?}", start_time.elapsed());

    srs_lagrange
}