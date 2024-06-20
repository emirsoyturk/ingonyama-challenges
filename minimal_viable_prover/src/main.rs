use lambdaworks_math::cyclic_group::IsGroup;
use lambdaworks_math::elliptic_curve::short_weierstrass::curves::bls12_381::default_types::FrField;
use lambdaworks_math::elliptic_curve::short_weierstrass::point::ShortWeierstrassProjectivePoint;
use lambdaworks_math::msm;
use lambdaworks_math::polynomial::Polynomial;
use lambdaworks_math::unsigned_integer::element::UnsignedInteger;
use lambdaworks_math::{
    elliptic_curve::{
        short_weierstrass::curves::bls12_381::curve::BLS12381Curve,
        traits::IsEllipticCurve,
    }, 
    field::element::FieldElement,
};

fn random_poly(degree: usize) -> Polynomial<Fr> {
    let coefficients: Vec<Fr> = (0..degree).map(|_| Fr::from(rand::random::<u64>())).collect();
    Polynomial::new(&coefficients)
}

fn idft(a: &[ShortWeierstrassProjectivePoint<BLS12381Curve>], degree: u64) -> Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>> {
    let mut srs_lagrange = vec![];
    for i in 0..degree {
        let mut sum: ShortWeierstrassProjectivePoint<BLS12381Curve> = ShortWeierstrassProjectivePoint::neutral_element();
        let mut power = Fr::one();
        a.iter().for_each(|p| {
            sum = sum.operate_with(&p.operate_with_self(power.representative()));
            power = &power * Fr::from(i);
        });

        srs_lagrange.push(sum);
    }

    srs_lagrange
}


type Fr = FieldElement<FrField>;
// type Fq = FieldElement<BLS12381PrimeField>;

struct StructuredReferenceString {    
    srs_lagrange: Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>>,
    random_poly: Polynomial<Fr>,
}


impl StructuredReferenceString {
    fn new(degree: usize) -> Self {
        let random = rand::random::<u64>();
        // generate random alpha    
        let alpha = Fr::from(random);

        //TODO: Randomize the generator
        let g = BLS12381Curve::generator();

        // compute the SRS in monomial basis
        let srs_monomial: Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>> = 
            (0..degree)
                .map(|exp| 
                    g.operate_with_self(
                        alpha.pow(exp as u64).representative()
                    )
                )
                .collect();

        let srs_lagrange = idft(&srs_monomial, degree as u64);

        StructuredReferenceString { 
            srs_lagrange,
            random_poly: random_poly(degree),
        }
    }
}


struct Prover {
    srs: StructuredReferenceString,
    witness: Polynomial<Fr>,
}

fn hadamard_product(a: &[Fr], b: &[Fr]) -> Vec<Fr> {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
}

impl Prover {
    fn compute_commitment(&self) -> ShortWeierstrassProjectivePoint<BLS12381Curve> {
        // Convert the vector of fi into a vector of length 2n using an FFT
        let fft_evals = Polynomial::evaluate_fft::<FrField>(&self.witness, 2, Option::None).unwrap();
        // Compute the commitment
        let hadamard_product = hadamard_product(&self.srs.random_poly.coefficients, &fft_evals);
        let c: Vec<UnsignedInteger<4>> = hadamard_product.iter().map(|x| UnsignedInteger { limbs: x.representative().limbs }).collect();
        
        msm::naive::msm::<UnsignedInteger<4>, ShortWeierstrassProjectivePoint<BLS12381Curve>>(&c, &self.srs.srs_lagrange).unwrap()
    }
}


fn main() {
    let degree = 3;  // Example degree, adjust as needed
    let srs = StructuredReferenceString::new(degree);
    let prover = Prover { srs, witness: random_poly(degree)};
    println!("{:?}", prover.compute_commitment());
}
