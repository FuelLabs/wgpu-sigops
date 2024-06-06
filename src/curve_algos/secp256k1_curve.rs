use crate::curve_algos::coords::{JacobianXYZ, ProjectiveXYZ};
use ark_ec::AffineRepr;
use ark_ff::{Field, One, Zero};
use ark_secp256k1::{Affine, Fq, Projective};

pub fn jacobian_coords(pt: &Projective) -> (Fq, Fq, Fq) {
    let x = pt.x;
    let y = pt.y;
    let z = pt.z;
    (x, y, z)
}

pub fn is_projective_zero(pt: &ProjectiveXYZ<Fq>) -> bool {
    pt.x == Fq::zero() && pt.y == Fq::one() && pt.z == Fq::zero()
}

pub fn is_jacobian_zero(pt: &JacobianXYZ<Fq>) -> bool {
    pt.x == Fq::one() && pt.y == Fq::one() && pt.z == Fq::zero()
}

pub fn jacobian_to_normalised_projective(pt: &JacobianXYZ<Fq>) -> ProjectiveXYZ<Fq> {
    if is_jacobian_zero(pt) {
        // If the point is infinity, return infinity (0:1:0)
        ProjectiveXYZ {
            x: Fq::zero(),
            y: Fq::one(),
            z: Fq::zero(),
        }
    } else if pt.z == Fq::one() {
        // If the point is already normalised
        ProjectiveXYZ {
            x: pt.x,
            y: pt.y,
            z: pt.z,
        }
    } else {
        // Perform the conversion
        let zinv = pt.z.inverse().unwrap();
        let zinv_squared = zinv.square();
        let x = pt.x * &zinv_squared;
        let y = pt.y * &(zinv_squared * &zinv);
        ProjectiveXYZ { x, y, z: pt.z }
    }
}

/// https://www.hyperelliptic.org/EFD/g1p/auto-shortw-projective.html#addition-add-2007-bl
/// Unsafe as it does not work with the point at infinity!
/// Cost: 16M
pub fn projective_add_2007_bl_unsafe(
    a: &ProjectiveXYZ<Fq>,
    b: &ProjectiveXYZ<Fq>,
) -> ProjectiveXYZ<Fq> {
    // TODO: is the correct way to check that a Projective point is the point at infinity just to
    // check that x and z are zero, even if y is not 1?
    if a.x == Fq::zero() && a.z == Fq::zero() {
        return b.clone();
    } else if b.x == Fq::zero() && b.z == Fq::zero() {
        return a.clone();
    }

    let x1: Fq = a.x;
    let y1: Fq = a.y;
    let z1: Fq = a.z;
    let x2: Fq = b.x;
    let y2: Fq = b.y;
    let z2: Fq = b.z;

    let u1 = &x1 * &z2;
    let u2 = &x2 * &z1;
    let s1 = &y1 * &z2;
    let s2 = &y2 * &z1;
    let zz = &z1 * &z2;
    let t = &u1 + &u2;
    let tt = &t * &t;
    let m = &s1 + &s2;
    let u1u2 = &u1 * &u2;
    let r = &tt - &u1u2;
    let f = &zz * &m;
    let l = &m * &f;
    let ll = &l * &l;
    let ttll = &tt + &ll;
    let tl = &t + &l;
    let tl2 = &tl * &tl;
    let g = &tl2 - &ttll;
    let r2 = &r * &r;
    let r22 = &r2 + &r2;
    let w = &r22 - &g;
    let f2 = &f + &f;
    let x3 = &f2 * &w;
    let ll2 = &ll + &ll;
    let w2 = &w + &w;
    let g2w = &g - &w2;
    let rg2w = &r * &g2w;
    let y3 = &rg2w - &ll2;
    let ff = &f * &f;
    let f4 = &f2 + &f2;
    let z3 = &f4 * &ff;

    ProjectiveXYZ {
        x: x3,
        y: y3,
        z: z3,
    }
}

/// https://www.hyperelliptic.org/EFD/g1p/auto-shortw-projective.html#doubling-dbl-2007-bl
/// Cost: 10M
pub fn projective_dbl_2007_bl_unsafe(x: &ProjectiveXYZ<Fq>) -> ProjectiveXYZ<Fq> {
    let x1 = x.x;
    let y1 = x.y;
    let z1 = x.z;

    let xx = &x1 * &x1;
    let xx2 = &xx + &xx;
    let w = &xx2 + &xx;
    let y1z1 = &y1 * &z1;
    let s = &y1z1 + &y1z1;
    let ss = &s * &s;
    let sss = &s * &ss;
    let r = &y1 * &s;
    let rr = &r * &r;
    let xxrr = &xx + &rr;
    let x1r = &x1 + &r;
    let x1r2 = &x1r * &x1r;
    let b = &x1r2 - &xxrr;
    let b2 = &b + &b;
    let w2 = &w * &w;
    let h = &w2 - &b2;
    let x3 = &h * &s;
    let rr2 = &rr + &rr;
    let bh = &b - &h;
    let wbh = &w * &bh;
    let y3 = &wbh - &rr2;
    let z3 = sss;
    /*
    XX = X1^2
    ZZ = Z1^2
    w = a*ZZ+3*XX
    s = 2*Y1*Z1
    ss = s^2
    sss = s*ss
    R = Y1*s
    RR = R^2
    B = (X1+R)^2-XX-RR
    h = w^2-2*B
    X3 = h*s
    Y3 = w*(B-h)-2*RR
    Z3 = sss
    */

    ProjectiveXYZ {
        x: x3,
        y: y3,
        z: z3,
    }
}

/// http://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-add-2007-bl
/// ark-ec-0.4.2/src/models/short_weierstrass/group.rs
/// Unsafe as it does not work with the point at infinity!
/// Cost: 16M
pub fn jacobian_add_2007_bl_unsafe(a: &Projective, b: &Projective) -> JacobianXYZ<Fq> {
    let (x1, y1, z1) = jacobian_coords(a);
    let (x2, y2, z2) = jacobian_coords(b);

    let z1z1 = z1 * z1;
    let z2z2 = z2 * z2;
    let u1 = x1 * &z2z2;
    let u2 = x2 * &z1z1;
    let s1 = y1 * z2 * &z2z2;
    let s2 = y2 * z1 * &z1z1;
    let h = &u2 - &u1;
    let h2 = &h + &h;
    let i = &h2 * &h2;
    let j = &h * &i;

    let s2s1 = &s2 - &s1;
    let r = &s2s1 + &s2s1;
    let v = &u1 * &i;
    let v2 = &v + &v;
    let r2 = &r * &r;
    let jv2 = &j + &v2;
    let x3 = &r2 - &jv2;

    let vx3 = &v - &x3;
    let rvx3 = &r * &vx3;
    let s12 = &s1 + &s1;
    let s12j = &s12 * &j;
    let y3 = &rvx3 - &s12j;

    // Z3 = 2 * Z1 * Z2 * H is faster
    let z1z2 = z1 * z2;
    let z1z2h = &z1z2 * &h;
    let z3 = &z1z2h + &z1z2h;

    JacobianXYZ {
        x: x3,
        y: y3,
        z: z3,
    }
}

/// ark-ec-0.4.2/src/models/short_weierstrass/group.rs
/// http://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
/// Cost: 7M
pub fn jacobian_dbl_2009_l(x1: &Projective) -> JacobianXYZ<Fq> {
    let (x, y, z) = jacobian_coords(x1);

    let a = &x * &x;
    let b = &y * &y;
    let c = &b * &b;
    let x1b = &x + &b;
    let x1b2 = &x1b * &x1b;
    let ac = &a + &c;
    let x1b2ac = &x1b2 - &ac;
    let d = &x1b2ac + &x1b2ac;
    let a2 = &a + &a;
    let e = &a2 + &a;
    let f = &e * &e;
    let d2 = &d + &d;
    let x3 = &f - &d2;
    let c2 = &c + &c;
    let c4 = &c2 + &c2;
    let c8 = &c4 + &c4;
    let dx3 = &d - &x3;
    let edx3 = &e * &dx3;
    let y3 = &edx3 - &c8;
    let y1z1 = &y * &z;
    let z3 = &y1z1 + &y1z1;

    JacobianXYZ {
        x: x3,
        y: y3,
        z: z3,
    }
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
        return ProjectiveXYZ {
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

#[cfg(test)]
pub mod tests {
    use crate::curve_algos::secp256k1_curve as curve;
    use ark_ec::{AffineRepr, CurveGroup};
    use ark_ff::Zero;
    use ark_secp256k1::{Affine, Fq, Fr, Projective};
    use std::ops::Mul;

    #[test]
    pub fn test_projectivexyz_eq() {
        let g = Affine::generator();
        let a: Affine = g.mul(Fr::from(2u32)).into_affine();
        let a_proj = curve::affine_to_projectivexyz(&a);
        assert!(a_proj == a_proj);

        let b: Affine = g.mul(Fr::from(3u32)).into_affine();
        let b_proj = curve::affine_to_projectivexyz(&b);
        assert!(a_proj != b_proj);
    }

    #[test]
    pub fn test_projective_add_2007_bl_unsafe() {
        // Test with different points
        let g = Affine::generator();
        let a: Affine = g.mul(Fr::from(2u32)).into_affine();
        let b: Affine = g.mul(Fr::from(3u32)).into_affine();

        let a_proj = curve::affine_to_projectivexyz(&a);
        let b_proj = curve::affine_to_projectivexyz(&b);

        let expected = a + b;
        let sum = curve::projective_add_2007_bl_unsafe(&a_proj, &b_proj);

        let sum_affine = curve::projectivexyz_to_affine(&sum);

        assert_eq!(sum_affine, expected.into_affine());

        // Test with the same point
        let a: Affine = g.mul(Fr::from(2u32)).into_affine();
        let b = a.clone();
        let a_proj = curve::affine_to_projectivexyz(&a);
        let b_proj = curve::affine_to_projectivexyz(&b);
        let sum = curve::projective_add_2007_bl_unsafe(&a_proj, &b_proj);
        let expected = a + b;
        let sum_affine = curve::projectivexyz_to_affine(&sum);

        assert_eq!(sum_affine, expected.into_affine());

        // Test with the point at infinity
        let a: Affine = g.mul(Fr::from(2u32)).into_affine();
        let b = Affine::zero();
        let a_proj = curve::affine_to_projectivexyz(&a);
        let b_proj = curve::affine_to_projectivexyz(&b);
        let sum = curve::projective_add_2007_bl_unsafe(&a_proj, &b_proj);
        let expected = a + b;
        let sum_affine = curve::projectivexyz_to_affine(&sum);
        assert_eq!(sum_affine, expected.into_affine());
    }

    #[test]
    pub fn test_projective_dbl_2007_bl_unsafe() {
        let g = Affine::generator();
        let a: Affine = g.mul(Fr::from(2u32)).into_affine();
        let a_proj = curve::affine_to_projectivexyz(&a);

        let expected = a + a;
        let sum = curve::projective_dbl_2007_bl_unsafe(&a_proj);

        let sum_affine = curve::projectivexyz_to_affine(&sum);

        assert!(sum_affine == expected.into_affine());

        // Test with the point at infinity
        let z = Affine::zero();
        let z_proj = curve::affine_to_projectivexyz(&z);
        let sum = curve::projective_dbl_2007_bl_unsafe(&z_proj);

        // Should not be valid
        assert_eq!(sum.x, Fq::zero());
        assert_eq!(sum.y, Fq::zero());
        assert_eq!(sum.z, Fq::zero());
    }

    #[test]
    pub fn test_jacobian_add_2007_bl_unsafe() {
        // Generate 2 different affine points
        let g = Affine::generator();
        let a: Projective = g.mul(Fr::from(2u32));
        let b: Projective = g.mul(Fr::from(3u32));

        // Compute the sum in Jacobian form
        let result = curve::jacobian_add_2007_bl_unsafe(&a, &b);
        let result_proj = Projective::new(result.x, result.y, result.z);
        let result_np = curve::jacobian_to_normalised_projective(&result);

        // Compute the sum in affine form using Arkworks
        let expected_sum = a + b;

        assert!(result_proj == expected_sum);

        let expected_sum_affine = expected_sum.into_affine();
        assert!(result_np.x == expected_sum_affine.x && result_np.y == expected_sum_affine.y);

        // Test with the point at infinity
        let paf: Projective = g.mul(Fr::from(0u32));
        let result = curve::jacobian_add_2007_bl_unsafe(&a, &paf);
        let result_proj = Projective::new(result.x, result.y, result.z);
        assert!(result_proj != a);

        let result = curve::jacobian_add_2007_bl_unsafe(&paf, &a);
        let result_proj = Projective::new(result.x, result.y, result.z);
        assert!(result_proj != a);

        // Test with the same point
        let a: Projective = g.mul(Fr::from(2u32));
        let b = a.clone();

        // Should return (0, 0, 0)
        let result = curve::jacobian_add_2007_bl_unsafe(&a, &b);
        assert_eq!(result.x, Fq::zero());
        assert_eq!(result.y, Fq::zero());
        assert_eq!(result.z, Fq::zero());
    }

    #[test]
    pub fn test_jacobian_dbl_2009_l() {
        let g = Affine::generator();
        let a: Projective = g.mul(Fr::from(2u32));
        let result = curve::jacobian_dbl_2009_l(&a);
        let result_proj = Projective::new(result.x, result.y, result.z);
        let result_np = curve::jacobian_to_normalised_projective(&result);

        let expected_sum = &a + &a;

        assert!(result_proj == expected_sum);

        let expected_sum_affine = expected_sum.into_affine();
        assert!(result_np.x == expected_sum_affine.x && result_np.y == expected_sum_affine.y);

        let paf: Projective = g.mul(Fr::from(0u32));
        let result = curve::jacobian_dbl_2009_l(&paf);
        let result_proj = Projective::new(result.x, result.y, result.z);
        assert!(result_proj == paf);
    }
}
