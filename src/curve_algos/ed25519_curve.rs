use crate::curve_algos::coords::ETEProjective;
use ark_ec::models::twisted_edwards::TECurveConfig;
use ark_ed25519::{EdwardsAffine, EdwardsConfig, Fq};
use ark_ff::{BigInteger, One, PrimeField};

pub fn affine_to_projective(point: &EdwardsAffine) -> ETEProjective<Fq> {
    ETEProjective {
        x: point.x,
        y: point.y,
        t: point.x * point.y,
        z: Fq::one(),
    }
}

pub fn projective_to_affine(point: &ETEProjective<Fq>) -> EdwardsAffine {
    // From https://docs.rs/ark-ec/0.4.0/src/ark_ec/models/twisted_edwards/group.rs.html#207:
    // A projective curve element (x, y, t, z) is normalized
    // to its affine representation, by the conversion
    // (x, y, t, z) -> (x/z, y/z, t/z, 1)
    let mut zs = vec![point.z];
    ark_ff::batch_inversion(&mut zs);
    let z_inv = zs[0];
    let x = point.x * z_inv;
    let y = point.y * z_inv;

    EdwardsAffine::new(x, y)
}

/// https://www.hyperelliptic.org/EFD/g1p/auto-twisted-extended-1.html#addition-add-2008-hwcd-3
pub fn ete_add_2008_hwcd_3(a: &ETEProjective<Fq>, b: &ETEProjective<Fq>) -> ETEProjective<Fq> {
    let x1: Fq = a.x;
    let y1: Fq = a.y;
    let t1: Fq = a.t;
    let z1: Fq = a.z;
    let x2: Fq = b.x;
    let y2: Fq = b.y;
    let t2: Fq = b.t;
    let z2: Fq = b.z;

    let two = Fq::from(2u32);
    let k = EdwardsConfig::COEFF_D * &two;

    let a = (&y1 - &x1) * (&y2 - &x2);
    let b = (&y1 + &x1) * (&y2 + &x2);
    let c = &t1 * &k * &t2;
    let d = &z1 * &two * &z2;
    let e = &b - &a;
    let f = &d - &c;
    let g = &d + &c;
    let h = &b + &a;
    let x3 = &e * &f;
    let y3 = &g * &h;
    let t3 = &e * &h;
    let z3 = &f * &g;

    ETEProjective {
        x: x3,
        y: y3,
        t: t3,
        z: z3,
    }
}

/// https://www.hyperelliptic.org/EFD/g1p/auto-twisted-extended-1.html#doubling-dbl-2008-hwcd
pub fn ete_dbl_2008_hwcd(a: &ETEProjective<Fq>) -> ETEProjective<Fq> {
    let x1: Fq = a.x;
    let y1: Fq = a.y;
    let z1: Fq = a.z;

    let p = Fq::MODULUS;
    let p = Fq::from_be_bytes_mod_order(&p.to_bytes_be());

    let a = &x1 * &x1;
    let b = &y1 * &y1;
    let z1z1 = &z1 * &z1;
    let c = &z1z1 + &z1z1;
    let d = &p - &a;
    let x1y1 = &x1 + &y1;
    let x1y12 = &x1y1 * &x1y1;
    let ab = &a + &b;
    let e = &x1y12 - &ab;
    let g = &d + &b;
    let f = &g - &c;
    let h = &d - &b;
    let x3 = &e * &f;
    let y3 = &g * &h;
    let t3 = &e * &h;
    let z3 = &f * &g;

    ETEProjective {
        x: x3,
        y: y3,
        t: t3,
        z: z3,
    }
}

#[cfg(test)]
pub mod tests {
    use crate::curve_algos::ed25519_curve as curve;
    use ark_ec::{AffineRepr, CurveGroup};
    use ark_ed25519::{EdwardsAffine, Fr};
    use std::ops::Mul;

    #[test]
    pub fn test_ete_add_2008_hwcd_3() {
        // Test with different points
        let g = EdwardsAffine::generator();
        let a: EdwardsAffine = g.mul(Fr::from(2u32)).into_affine();
        let b: EdwardsAffine = g.mul(Fr::from(3u32)).into_affine();

        let a_proj = curve::affine_to_projective(&a);
        let b_proj = curve::affine_to_projective(&b);

        let expected = a + b;
        let sum = curve::ete_add_2008_hwcd_3(&a_proj, &b_proj);

        let sum_affine = curve::projective_to_affine(&sum);

        assert_eq!(sum_affine, expected.into_affine());

        // Test with the same point
        let expected = a + a;
        let sum = curve::ete_add_2008_hwcd_3(&a_proj, &a_proj);

        let sum_affine = curve::projective_to_affine(&sum);

        assert_eq!(sum_affine, expected.into_affine());

        // Test with the point at infinity
        let b = EdwardsAffine::zero();
        let b_proj = curve::affine_to_projective(&b);

        let expected = a + b;
        let sum = curve::ete_add_2008_hwcd_3(&a_proj, &b_proj);

        let sum_affine = curve::projective_to_affine(&sum);

        assert_eq!(sum_affine, expected.into_affine());
    }

    #[test]
    pub fn test_ete_dbl_2008_hwcd() {
        let g = EdwardsAffine::generator();
        let a: EdwardsAffine = g.mul(Fr::from(2u32)).into_affine();
        let a_proj = curve::affine_to_projective(&a);

        let expected = a + a;
        let sum = curve::ete_dbl_2008_hwcd(&a_proj);

        let sum_affine = curve::projective_to_affine(&sum);

        assert_eq!(sum_affine, expected.into_affine());

        // Test with the point at infinity
        let a = EdwardsAffine::zero();
        let a_proj = curve::affine_to_projective(&a);

        let expected = a + a;
        let sum = curve::ete_dbl_2008_hwcd(&a_proj);

        let sum_affine = curve::projective_to_affine(&sum);

        assert_eq!(sum_affine, expected.into_affine());
    }
}
