#[cfg(test)]
pub mod tests {
    use crate::curve_algos::{fixed_base_ec_mul, double_and_add};
    use crate::curve_algos::precompute::precompute_table;
    use crate::curve_algos::secp256k1_curve::glv_constants;
    use num_bigint::{BigUint, RandomBits};
    use ark_secp256k1::{Affine, Projective, Fr};
    use ark_ff::PrimeField;
    use ark_ec::{AffineRepr, CurveGroup, Group};
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::ops::{Mul, Neg, Shr};

    #[test]
    pub fn test_fixed_base_mul() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let w = 4u32;

        let pt = Projective::generator();

        let table = precompute_table::<Projective>(pt, w);

        for _ in 0..1000 {
            let scalar: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let scalar = Fr::from_be_bytes_mod_order(&scalar.to_bytes_be());

            let result = double_and_add(pt, scalar);
            let result2 = fixed_base_ec_mul::<Projective, Fr>(&table, scalar, w);
            let expected = pt.mul(scalar).into_affine();

            assert_eq!(result.into_affine(), expected);
            assert_eq!(result2.into_affine(), expected);
        }
    }

    #[test]
    pub fn test_secp256k1_glv() {
        // Constants
        let n = crate::moduli::secp256k1_fr_modulus_biguint();
        let half_n = &n / BigUint::from(2u32);
        let half_n = Fr::from(half_n);

        let (beta, a1, b1, a2, b2, g1, g2) = glv_constants();
        let mut rng = ChaCha8Rng::seed_from_u64(2);

        for _ in 0..1000 {
            // Generate a random scalar k
            let k: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &n;

            // Split k into k1 and k2
            let kg1 = &k * &g1;
            let kg2 = &k * &g2;
            let c1 = kg1.shr(384u32);
            let c2 = kg2.shr(384u32);

            let c1a1 = &c1 * &a1;
            let c2a2 = &c2 * &a2;
            let c1a1c2a2 = &c1a1 + &c2a2;

            let c2b2 = &c2 * &b2;
            let neg_c1 = &n - &c1;
            let neg_c1b1 = &neg_c1 * &b1;

            let mut k1 = Fr::from(&k - &c1a1c2a2);
            let mut k2 = Fr::from(&neg_c1b1 - &c2b2);

            // Generate a random point
            let r: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &n;
            let pt = Affine::generator().mul(Fr::from(r.clone())).into_affine();
            let mut p0 = pt;

            // Map pt to pt_prime
            let mut p1 = Affine::new(beta * pt.x, pt.y);

            // Normalise k1 and k2 to roughly half the bitlength of the scalar field
            if k1 > half_n {
                k1 = k1.neg();
                p0 = p0.neg();
            }

            if k2 > half_n {
                k2 = k2.neg();
                p1 = p1.neg();
            }

            let expected = pt.mul(Fr::from(k));

            // In production, use the Strauss-Shamir trick, which is more efficient.
            let result = p0.mul(k1) + p1.mul(k2);

            assert_eq!(result, expected);
        }
    }
}
