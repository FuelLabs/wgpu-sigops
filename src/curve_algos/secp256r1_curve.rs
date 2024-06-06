use crate::curve_algos::coords::ProjectiveXYZ;
use ark_ec::models::short_weierstrass::SWCurveConfig;
use ark_ec::AffineRepr;
use ark_ff::{Field, One, Zero};
use ark_secp256r1::{Affine, Config, Fq};

pub fn is_projective_zero(pt: &ProjectiveXYZ<Fq>) -> bool {
    pt.x == Fq::zero() && pt.y == Fq::one() && pt.z == Fq::zero()
}

pub fn projectivexyz_to_affine(point: &ProjectiveXYZ<Fq>) -> Affine {
    let x = point.x;
    let y = point.y;
    let z = point.z;

    let zinv = z.inverse().unwrap();
    Affine::new(x * zinv, y * zinv)
}

pub fn affine_to_projectivexyz(point: &Affine) -> ProjectiveXYZ<Fq> {
    if point.is_zero() {
        return ProjectiveXYZ::<Fq> {
            x: Fq::zero(),
            y: Fq::one(),
            z: Fq::zero(),
        };
    }
    ProjectiveXYZ {
        x: point.x,
        y: point.y,
        z: Fq::one(),
    }
}

/// https://www.hyperelliptic.org/EFD/g1p/auto-shortw-projective-3.html#addition-add-2015-rcb
pub fn projective_add_2015_rcb_unsafe(
    pt_a: &ProjectiveXYZ<Fq>,
    pt_b: &ProjectiveXYZ<Fq>,
) -> ProjectiveXYZ<Fq> {
    // TODO: is the correct way to check that a Projective point is the point at infinity just to
    // check that x and z are zero, even if y is not 1?
    if pt_a.x == Fq::zero() && pt_a.z == Fq::zero() {
        return pt_b.clone();
    } else if pt_b.x == Fq::zero() && pt_b.z == Fq::zero() {
        return pt_a.clone();
    }

    let a = Config::COEFF_A;
    let b = Config::COEFF_B;
    let b3 = b * Fq::from(3u32);

    let x1: Fq = pt_a.x;
    let y1: Fq = pt_a.y;
    let z1: Fq = pt_a.z;
    let x2: Fq = pt_b.x;
    let y2: Fq = pt_b.y;
    let z2: Fq = pt_b.z;

    let t0 = &x1 * &x2;
    let t1 = &y1 * &y2;
    let t2 = &z1 * &z2;
    let t3 = &x1 + &y1;
    let t4 = &x2 + &y2;
    let t3 = &t3 * &t4;
    let t4 = &t0 + &t1;
    let t3 = &t3 - &t4;
    let t4 = &x1 + &z1;
    let t5 = &x2 + &z2;
    let t4 = &t4 * &t5;
    let t5 = &t0 + &t2;
    let t4 = &t4 - &t5;
    let t5 = &y1 + &z1;
    let x3 = &y2 + &z2;
    let t5 = &t5 * &x3;
    let x3 = &t1 + &t2;
    let t5 = &t5 - &x3;
    let z3 = &a * &t4;
    let x3 = &b3 * &t2;
    let z3 = &x3 + &z3;
    let x3 = &t1 - &z3;
    let z3 = &t1 + &z3;
    let y3 = &x3 * &z3;
    let t1 = &t0 + &t0;
    let t1 = &t1 + &t0;
    let t2 = &a * &t2;
    let t4 = &b3 * &t4;
    let t1 = &t1 + &t2;
    let t2 = &t0 - &t2;
    let t2 = &a * &t2;
    let t4 = &t4 + &t2;
    let t0 = &t1 * &t4;
    let y3 = &y3 + &t0;
    let t0 = &t5 * &t4;
    let x3 = &t3 * &x3;
    let x3 = &x3 - &t0;
    let t0 = &t3 * &t1;
    let z3 = &t5 * &z3;
    let z3 = &z3 + &t0;

    ProjectiveXYZ {
        x: x3,
        y: y3,
        z: z3,
    }
}

/// https://www.hyperelliptic.org/EFD/g1p/auto-shortw-projective-3.html#doubling-dbl-2015-rcb
pub fn projective_dbl_2015_rcb(x: &ProjectiveXYZ<Fq>) -> ProjectiveXYZ<Fq> {
    let x1 = x.x;
    let y1 = x.y;
    let z1 = x.z;

    let a = Config::COEFF_A;
    let b = Config::COEFF_B;
    let b3 = b * Fq::from(3u32);

    let t0 = &x1 * &x1;
    let t1 = &y1 * &y1;
    let t2 = &z1 * &z1;
    let t3 = &x1 * &y1;
    let t3 = &t3 + &t3;
    let z3 = &x1 * &z1;
    let z3 = &z3 + &z3;
    let x3 = &a * &z3;
    let y3 = &b3 * &t2;
    let y3 = &x3 + &y3;
    let x3 = &t1 - &y3;
    let y3 = &t1 + &y3;
    let y3 = &x3 * &y3;
    let x3 = &t3 * &x3;
    let z3 = &b3 * &z3;
    let t2 = &a * &t2;
    let t3 = &t0 - &t2;
    let t3 = &a * &t3;
    let t3 = &t3 + &z3;
    let z3 = &t0 + &t0;
    let t0 = &z3 + &t0;
    let t0 = &t0 + &t2;
    let t0 = &t0 * &t3;
    let y3 = &y3 + &t0;
    let t2 = &y1 * &z1;
    let t2 = &t2 + &t2;
    let t0 = &t2 * &t3;
    let x3 = &x3 - &t0;
    let z3 = &t2 * &t1;
    let z3 = &z3 + &z3;
    let z3 = &z3 + &z3;

    ProjectiveXYZ {
        x: x3,
        y: y3,
        z: z3,
    }
}

#[cfg(test)]
pub mod tests {
    use crate::curve_algos::secp256r1_curve as curve;
    use ark_ec::AffineRepr;
    use ark_ec::CurveGroup;
    use ark_ff::{One, Zero};
    use ark_secp256r1::{Affine, Fq, Fr};
    use std::ops::Mul;

    #[test]
    pub fn test_projective_add_2015_rcb_unsafe() {
        // Test with different points
        let g = Affine::generator();
        let a: Affine = g.mul(Fr::from(2u32)).into_affine();
        let b: Affine = g.mul(Fr::from(3u32)).into_affine();
        let expected = a + b;

        let a_proj = curve::affine_to_projectivexyz(&a);
        let b_proj = curve::affine_to_projectivexyz(&b);
        let sum = curve::projective_add_2015_rcb_unsafe(&a_proj, &b_proj);
        let sum_affine = curve::projectivexyz_to_affine(&sum);

        assert_eq!(sum_affine, expected);

        // Test with the same point
        let expected = a + a;
        let sum = curve::projective_add_2015_rcb_unsafe(&a_proj, &a_proj);
        let sum_affine = curve::projectivexyz_to_affine(&sum);
        assert_eq!(sum_affine, expected);

        // Test with the point at infinity
        let z: Affine = Affine::zero();
        let expected = a + z;
        let z_proj = curve::affine_to_projectivexyz(&z);
        let sum = curve::projective_add_2015_rcb_unsafe(&a_proj, &z_proj);
        let sum_affine = curve::projectivexyz_to_affine(&sum);
        assert_eq!(sum_affine, expected);
    }

    #[test]
    pub fn test_projective_dbl_2015_rcb() {
        let g = Affine::generator();
        let a: Affine = g.mul(Fr::from(2u32)).into_affine();
        let a_proj = curve::affine_to_projectivexyz(&a);

        let expected = a + a;
        let sum = curve::projective_dbl_2015_rcb(&a_proj);

        let sum_affine = curve::projectivexyz_to_affine(&sum);

        assert!(sum_affine == expected.into_affine());

        // Test with the point at infinity
        let z = Affine::zero();
        let z_proj = curve::affine_to_projectivexyz(&z);
        let sum = curve::projective_dbl_2015_rcb(&z_proj);

        // Should be valid
        assert_eq!(sum.x, Fq::zero());
        assert_eq!(sum.y, Fq::one());
        assert_eq!(sum.z, Fq::zero());
    }
}
