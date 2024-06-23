use std::io::{Read, Write};

use lambdaworks_math::{
    cyclic_group::IsGroup, elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{
                curve::BLS12381Curve,
                default_types::FrField
            },
            point::ShortWeierstrassProjectivePoint,
        },
        traits::IsEllipticCurve,
    }, msm, polynomial::{pad_with_zero_coefficients_to_length, Polynomial}, traits::ByteConversion, unsigned_integer::element::UnsignedInteger
};
use minimal_viable_prover::{fft_over_ec, random_poly, Fq, Fr};
use rand::RngCore;
use rayon::prelude::*;

struct StructuredReferenceString {    
    srs_lagrange: Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>>,
    random_poly: Polynomial<Fr>,
}

impl StructuredReferenceString {
    fn read_srs_from_file(file_path: &str, degree: usize) -> Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>> {
        let file = std::fs::File::open(file_path).expect("Failed to open file");
        let mut reader = std::io::BufReader::new(file);
        let mut points = Vec::new();
        
        for _ in 0..degree {
            let mut buffer = [0u8; 48 * 3];
            reader.read_exact(&mut buffer).expect("Failed to read points");
            let x = Fq::from_bytes_le(&buffer[0..48]).unwrap();
            let y = Fq::from_bytes_le(&buffer[48..96]).unwrap();
            let z = Fq::from_bytes_le(&buffer[96..144]).unwrap();
            points.push(ShortWeierstrassProjectivePoint::new([x, y, z]));
        }
        points
    }

    fn write_srs_to_file(file_path: &str, srs_lagrange: &[ShortWeierstrassProjectivePoint<BLS12381Curve>]) {
        let file = std::fs::File::create(file_path).expect("Failed to create file");
        let mut writer = std::io::BufWriter::new(file);
        
        for point in srs_lagrange {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(point.x().to_bytes_le().as_ref());
            bytes.extend_from_slice(point.y().to_bytes_le().as_ref());
            bytes.extend_from_slice(point.z().to_bytes_le().as_ref());
            writer.write_all(&bytes).expect("Failed to write point");
        }
        writer.flush().expect("Failed to flush writer");
    }

    fn generate_srs(degree: usize) -> Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>> {
        let start_time = std::time::Instant::now();
        let mut rnd = rand::thread_rng();

        let alpha = Fr::from(rnd.next_u64());
        let alpha_powers: Vec<Fr> = (0..degree)
            .map(|i| alpha.pow(i as u64))
            .collect();

        let alpha2 = rnd.next_u64();
        let g = BLS12381Curve::generator().operate_with_self(alpha2);

        let srs_lagrange = fft_over_ec(&alpha_powers, g);
        
        println!("SRS generated in {:?}", start_time.elapsed());

        srs_lagrange
    }

    fn new(degree: usize, use_existing: bool) -> Self {
        let file_path = format!("srs_{degree}.dat");

        let srs_lagrange = if use_existing && std::path::Path::new(&file_path).exists() {
            println!("Loading SRS from file...");
            
            Self::read_srs_from_file(&file_path, degree)
        } else {
            println!("Generating new SRS...");
            let points = Self::generate_srs(degree);
            Self::write_srs_to_file(&file_path, &points);
            points
        };

        let random_poly = random_poly(degree);
        
        StructuredReferenceString { srs_lagrange, random_poly }
    }
}


struct Prover {
    srs: StructuredReferenceString,
    witness: Polynomial<Fr>,
}

fn hadamard_product(a: &[Fr], b: &[Fr]) -> Vec<Fr> {
    a.par_iter().zip(b.par_iter()).map(|(x, y)| x * y).collect()
}

impl Prover {
    fn compute_commitment(&self) -> ShortWeierstrassProjectivePoint<BLS12381Curve> {
        let mut witness = self.witness.clone();
        pad_with_zero_coefficients_to_length(&mut witness, 2 * self.witness.degree());
        let fft_evals = Polynomial::evaluate_fft::<FrField>(&witness, 1, Option::None).unwrap();
        let hadamard_product = hadamard_product(&self.srs.random_poly.coefficients, &fft_evals);
        let c: Vec<UnsignedInteger<4>> = hadamard_product.par_iter().map(|x| UnsignedInteger {
            limbs: x.representative().limbs,
        }).collect();

        msm::naive::msm::<UnsignedInteger<4>, ShortWeierstrassProjectivePoint<BLS12381Curve>>(&c, &self.srs.srs_lagrange).unwrap()
    }

    fn new(srs: StructuredReferenceString, degree: usize) -> Self {
        let witness = random_poly(degree);
        Prover { srs, witness }
    }
}


fn main() {
    let degree = (2u64.pow(17) * 2) as usize;
    let srs = StructuredReferenceString::new(degree, true);
    let prover = Prover::new(srs, degree);
    let start_time = std::time::Instant::now();
    let commitment = prover.compute_commitment();
    println!("Commitment is computed in {:?}\nCommitment: {:?}", start_time.elapsed(), commitment);
}
