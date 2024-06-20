use ark_secp256k1::{Affine, Projective};
use num_bigint::BigUint;
use ark_secp256k1::Fr;
use ark_ff::{BigInteger, PrimeField, Zero};
use ark_ec::Group;

pub fn fixed_base_ec_mul(
    table: &Vec<Affine>,
    scalar: Fr,
    w: u32,
) -> Projective {
    for i in 0..table.len() {
        println!("{}: {}", i, table[i]);
    }
    // Convert the scalar to bits
    let mut scalar_bits = vec![];
    let mut temp = BigUint::from_bytes_be(&scalar.into_bigint().to_bytes_be());

    let mut num_scalar_bits = 0;

    loop {
        if temp == BigUint::from(0u32) {
            break
        }

        scalar_bits.push(&temp % BigUint::from(2u32) == BigUint::from(1u32));
        num_scalar_bits += 1;

        temp /= BigUint::from(2u32);
    }

    // Scalar multiplication using the precomputed table
    let mut result = Projective::zero();
    let mut i = num_scalar_bits;
    while i > 0 {
        let mut bits = 0;
        for _ in 0..w {
            if i > 0 {
                i -= 1;
                bits <<= 1;
                if scalar_bits[i] {
                    bits |= 1;
                }
            }
        }
        for _ in 0..w {
            result = result.double();
        }
        if bits != 0 {
            result += table[bits as usize - 1];
        }
    }

    result
}

#[cfg(test)]
pub mod tests {
    use crate::curve_algos::secp256k1_mul::fixed_base_ec_mul;
    use crate::curve_algos::precompute::precompute_table;
    use num_bigint::{BigUint, RandomBits};
    use ark_secp256k1::{Projective, Fr};
    use ark_ff::{BigInteger, PrimeField, Zero};
    use ark_ec::{CurveGroup, Group};
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::ops::Mul;

    pub fn double_and_add(
        pt: Projective,
        scalar: Fr,
    ) -> Projective {
        // Convert the scalar to bits
        let scalar_bits = scalar.into_bigint().to_bits_be();

        let mut result = Projective::zero();
        let current = pt;

        let mut doublings = 0;
        let mut additions = 0;

        // Iterate over the bits of the scalar from the least significant bit to the most significant bit
        for bit in scalar_bits.iter() {
            // Double the point
            result.double_in_place();
            doublings += 1;

            // If the bit is 1, add the current point to the result
            if *bit {
                result += current;
                additions += 1;
            }
        }

        println!("bits: {}; doublings: {}; additions: {}", scalar_bits.len(), doublings, additions);
        result
    }

    #[test]
    pub fn test_fixed_base_mul() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let w = 4u32;

        let pt = Projective::generator();

        let table = precompute_table::<Projective>(pt, w);

        for _ in 0..1 {
            let scalar: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let scalar = Fr::from_be_bytes_mod_order(&scalar.to_bytes_be());

            let result = double_and_add(pt, scalar);
            let result2 = fixed_base_ec_mul(&table, scalar, w);
            let expected = pt.mul(scalar).into_affine();
            println!("{}", expected);

            assert_eq!(result, expected);
            assert_eq!(result2, expected);
        }
    }
}
