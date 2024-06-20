pub mod coords;
pub mod ed25519_curve;
pub mod ed25519_eddsa;
pub mod secp256k1_curve;
pub mod secp256k1_ecdsa;
pub mod secp256k1_mul;
pub mod secp256r1_curve;
pub mod secp256r1_ecdsa;
pub mod secp256r1_mul;
pub mod precompute;

use num_bigint::BigUint;
use ark_ff::{BigInteger, PrimeField};
use ark_ec::CurveGroup;

pub fn fixed_base_ec_mul<P: CurveGroup, R: PrimeField>(
    table: &Vec<P::Affine>,
    scalar: R,
    w: u32,
) -> P {
    //for i in 0..table.len() {
        //println!("{}: {}", i, table[i]);
    //}
    // Convert the scalar to bits
    let mut scalar_bits = vec![];
    let mut temp = BigUint::from_bytes_be(&scalar.into_bigint().to_bytes_be());

    let mut num_scalar_bits = 0;

    for _ in 0..256{
        scalar_bits.push(&temp % BigUint::from(2u32) == BigUint::from(1u32));
        num_scalar_bits += 1;

        temp /= BigUint::from(2u32);
    }

    // Scalar multiplication using the precomputed table
    let mut result = P::zero();
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

pub fn double_and_add<P: CurveGroup, R: PrimeField>(
    pt: P,
    scalar: R,
) -> P {
    // Convert the scalar to bits
    let scalar_bits = scalar.into_bigint().to_bits_be();

    assert_eq!(scalar_bits.len(), 256);

    let mut result = P::zero();
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

    if false {
        println!("bits: {}; doublings: {}; additions: {}", scalar_bits.len(), doublings, additions);
    }
    result
}

