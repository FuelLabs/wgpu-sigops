use ark_secp256k1::{Affine, Fq, Fr};
use ark_ec::{AffineRepr, CurveGroup};
use std::ops::{Mul, Neg};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[serial_test::serial]
#[tokio::test]
pub async fn test_secp256k1_glv() {
    // Constants
    let n = crate::moduli::secp256k1_fr_modulus_biguint();
    let half_n = &n / BigUint::from(2u32);
    let h = Fr::from(half_n);
    
    // Precomputed by Hal Finney
    //let lambda = BigUint::parse_bytes(b"5363ad4cc05c30e0a5261c028812645a122e22ea20816678df02967c1b23bd72", 16).unwrap();
    //let lambda = Fr::from(lambda);

    let beta = BigUint::parse_bytes(b"7ae96a2b657c07106e64479eac3434e99cf0497512f58995c1396c28719501ee", 16).unwrap();
    let beta = Fq::from(beta);

    let a1 = BigUint::parse_bytes(b"3086d221a7d46bcde86c90e49284eb15", 16).unwrap();
    let b1 = &n - BigUint::parse_bytes(b"e4437ed6010e88286f547fa90abfe4c3", 16).unwrap();
    let neg_b1 = &n - &b1;
    let a2 = BigUint::parse_bytes(b"114ca50f7a8e2f3f657c1108d9d44cfd8", 16).unwrap();
    let b2 = a1.clone();

    let mut rng = ChaCha8Rng::seed_from_u64(2);

    for _ in 0..1000 {
        // Generate a random scalar k
        let k: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &n;

        // Split k into k1 and k2
        let c1 = (&b2 * &k) / &n;
        let c2 = ((&neg_b1) * &k) / &n;

        let mut k1 = Fr::from(&k - &c1 * &a1 - &c2 * &a2);
        let mut k2 = Fr::from((&n - &c1) * &b1 - &c2 * &b2);

        // Generate a random point
        let r: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &n;
        let pt = Affine::generator().mul(Fr::from(r.clone())).into_affine();
        let mut p0 = pt;

        // Map pt to pt_prime
        let mut p1 = Affine::new(beta * pt.x, pt.y);

        // Normalise k1 and k2 to roughly half the bitlength of the scalar field
        if k1 > h {
            k1 = k1.neg();
            p0 = p0.neg();
        }

        if k2 > h {
            k2 = k2.neg();
            p1 = p1.neg();
        }

        let expected = pt.mul(Fr::from(k));

        // In production, use the Strauss-Shamir trick which is more efficient.
        let result = p0.mul(k1) + p1.mul(k2);

        assert_eq!(result, expected);
    }
}
