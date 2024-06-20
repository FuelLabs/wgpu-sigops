use crate::curve_algos::precompute::precompute_table;
use crate::curve_algos::coords::ProjectiveXYZ;
use crate::curve_algos::ed25519_curve::affine_to_projective;
use num_bigint::BigUint;
use crate::tests::{projectivexy_to_mont_limbs, eteprojective_to_xyt_mont_limbs};
use crate::curve_algos::{secp256k1_curve, secp256r1_curve};
use ark_ed25519::{EdwardsAffine, EdwardsProjective, Fq};
use ark_ec::AffineRepr;
use ark_ec::CurveGroup;
use ark_ff::{BigInteger, PrimeField};

pub const WINDOW_SIZE: u32 = 4;

pub fn generate_table<P: CurveGroup, Q: PrimeField>(
    log_limb_size: u32,
    affine_to_projectivexyz: fn (point: &P::Affine) -> ProjectiveXYZ<Q>,
) -> Vec<u32> {
    let g = P::Affine::generator();
    let p = BigUint::from_bytes_be(&Q::MODULUS.to_bytes_be());

    let table = precompute_table::<P>(g.into(), WINDOW_SIZE);

    //for i in 0..table.len() {
        //println!("i: {}, {}", i, table[i]);
    //}

    let mut table_limbs = vec![];
    for t in &table {
        let pt_xyz = affine_to_projectivexyz(t);
        table_limbs.extend(projectivexy_to_mont_limbs(&pt_xyz, &p, log_limb_size));
    }

    table_limbs
}

pub fn secp256k1_bases(
    log_limb_size: u32
) -> Vec<u32> {
    generate_table::<ark_secp256k1::Projective, ark_secp256k1::Fq>(
        log_limb_size,
        secp256k1_curve::affine_to_projectivexyz,
    )
}

pub fn secp256r1_bases(
    log_limb_size: u32
) -> Vec<u32> {
    generate_table::<ark_secp256r1::Projective, ark_secp256r1::Fq>(
        log_limb_size,
        secp256r1_curve::affine_to_projectivexyz,
    )
}

pub fn ed25519_bases(
    log_limb_size: u32
) -> Vec<u32> {
    let g = EdwardsAffine::generator();
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());

    let table = precompute_table::<EdwardsProjective>(g.into(), WINDOW_SIZE);

    let mut table_limbs = vec![];
    for t in &table {
        let pt_xytz = affine_to_projective(t);
        table_limbs.extend(eteprojective_to_xyt_mont_limbs(&pt_xytz, &p, log_limb_size));
    }

    table_limbs
}
