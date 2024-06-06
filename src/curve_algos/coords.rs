/// Arkworks supports Jacobian coordinates, which it describes as "projective". This struct, in
/// contrast, refers to projective coordinates (X, Y, Z) such that the affine coordinates x and y
/// are:
/// x=X/Z
/// y=Y/Z
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ProjectiveXYZ<F> {
    pub x: F,
    pub y: F,
    pub z: F,
}

/// This struct maps to Arkworks' Projective struct which are actually Jacobian coordinates, such
/// that the affine coordinates x and y are:
/// x = X/Z^3
/// y = Y/Z^3
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JacobianXYZ<F> {
    pub x: F,
    pub y: F,
    pub z: F,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ETEProjective<F> {
    pub x: F,
    pub y: F,
    pub t: F,
    pub z: F,
}
