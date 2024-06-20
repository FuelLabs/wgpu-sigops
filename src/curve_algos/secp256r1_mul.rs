#[cfg(test)]
pub mod tests {
    use crate::curve_algos::{fixed_base_ec_mul, double_and_add};
    use crate::curve_algos::precompute::precompute_table;
    use num_bigint::{BigUint, RandomBits};
    use ark_secp256r1::{Projective, Fr};
    use ark_ff::PrimeField;
    use ark_ec::{CurveGroup, Group};
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::ops::Mul;

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

            assert_eq!(result, expected);
            assert_eq!(result2, expected);
        }
    }
}
