use ark_ec::CurveGroup;

pub fn precompute_table<P: CurveGroup>(
    pt: P,
    w: u32,
) -> Vec<P::Affine> {
    // Precompute the lookup table
    let mut table: Vec<P::Affine> = Vec::new();
    let mut current = pt;
    table.push(current.into_affine());
    for _ in 1..(1 << w) {
        current += pt;
        table.push(current.into_affine());
    }

    table
}
