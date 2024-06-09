#[cfg(test)]
pub mod mont;
#[cfg(test)]
pub mod secp256k1_ecdsa;
#[cfg(test)]
pub mod secp256r1_ecdsa;
#[cfg(test)]
pub mod ed25519_eddsa;

pub fn compute_num_workgroups(
    num_threads: usize,
    workgroup_size: usize,
) -> (usize, usize, usize) {
    assert!(num_threads <= 256 * 256 * 256 * 64);
    // Assume that num_threads and workgroup_size are powers of 2, the number of workgroups per
    // dimension are powers of 2, and that the maximum number of X and Y workgroups per dimension
    // is less than or equal to 256, and the maximum number of Z workgroups is less than or equal
    // to 64
    if num_threads <= workgroup_size {
        return (1, 1, 1);
    }

    let triple = num_threads / workgroup_size;

    let (num_x_workgroups, num_y_workgroups, num_z_workgroups) = match triple {
        2 => (2, 1, 1),
        4 => (4, 1, 1),
        8 => (8, 1, 1),
        16 => (16, 1, 1),
        32 => (32, 1, 1),
        64 => (64, 1, 1),
        128 => (128, 1, 1),
        256 => (256, 1, 1),
        512 => (256, 2, 1),
        1024 => (256, 4, 1),
        2048 => (256, 8, 1),
        4096 => (256, 16, 1),
        8192 => (256, 32, 1),
        16384 => (256, 64, 1),
        32768 => (256, 128, 1),
        65536 => (256, 256, 1),
        131072 => (256, 256, 2),
        262144 => (256, 256, 4),
        524288 => (256, 256, 8),
        1048576 => (256, 256, 16),
        2097152 => (256, 256, 32),
        4194304 => (256, 256, 64),
        _ => unimplemented!()
    };

    assert_eq!(
        workgroup_size * num_x_workgroups * num_y_workgroups * num_z_workgroups,
        num_threads
    );
    (num_x_workgroups, num_y_workgroups, num_z_workgroups)
}

#[test]
pub fn test_compute_num_workgroups() {
    let workgroup_size = 256;
    for i in 0..23 {
        let num_threads = 2u32.pow(i) as usize;
        let _ = compute_num_workgroups(num_threads, workgroup_size);
    }
}
