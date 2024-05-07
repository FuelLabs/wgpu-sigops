use multiprecision::utils::calc_num_limbs;
use multiprecision::bigint;
use num_bigint::{ BigUint, RandomBits };
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use crate::shader::render_bytes_to_limbs_test;
use crate::tests::get_secp256k1_b;
use crate::gpu::{
    create_empty_sb,
    execute_pipeline,
    create_bind_group,
    create_sb_with_data,
    get_device_and_queue,
    create_command_encoder,
    create_compute_pipeline,
    finish_encoder_and_read_from_gpu,
};

#[serial_test::serial]
#[tokio::test]
pub async fn test_bytes_be_to_limbs_le_shader() {
    let mut rng = ChaCha8Rng::seed_from_u64(33);
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f", 16).unwrap();
    for log_limb_size in 11..16 {
        for _ in 0..10 {
            let val: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let bytes = val.to_bytes_be();

            do_test_bytes_be_to_limbs_le(&bytes, &p, log_limb_size).await;
        }
    }
}

pub async fn do_test_bytes_be_to_limbs_le(
    bytes: &Vec<u8>,
    p: &BigUint,
    log_limb_size: u32,
) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let expected = BigUint::from_bytes_be(&bytes);

    let _expected_limbs = bigint::from_biguint_le(&expected, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;

    let bytes_u32s: Vec<u32> = bytemuck::cast_slice(&bytes).to_vec();
    let bytes_buf = create_sb_with_data(&device, &bytes_u32s);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);

    let source = render_bytes_to_limbs_test("src/wgsl/", "bytes_be_to_limbs_le_tests.wgsl", &p, &get_secp256k1_b(), log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_bytes_be_to_limbs_le");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&bytes_buf, &result_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(
        &results[0][0..num_limbs].to_vec(),
        num_limbs,
        log_limb_size,
    );

    assert_eq!(result, expected);
}

#[test]
pub fn test_bytes_be_to_limbs_le() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    for log_limb_size in 11..16 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for _ in 0..100 {
            //let h = "f88a02175b81b28a82853361fef8f2d8ff5bb5c873a4ba53f01bdff367b10cf0";
            //let bytes = hex::decode(&h).unwrap();
            //let val = BigUint::from_bytes_be(&bytes);
            let val: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));

            let mut bytes = val.to_bytes_be();
            while bytes.len() < 32 {
                bytes.insert(0, 0);
            }
            let expected_limbs = bigint::from_biguint_le(&val, num_limbs, log_limb_size);
            let mut limbs = Vec::with_capacity(num_limbs);

            for i in 0..(256u32 / log_limb_size) {
                let x = (log_limb_size * (i + 1) as u32) % 8;
                let y_p_z = log_limb_size - x;
                let mut y = 0u32;
                let mut z = 0u32;

                if y_p_z > 8 {
                    y = 8;
                    z = log_limb_size - x - y;
                } else {
                    y = log_limb_size - x - y;
                }

                let bx = (log_limb_size * (i + 1) as u32) / 8;

                if bx > 31 {
                    break;
                }

                let by = bx - 1;
                let bz = if z == 0 {
                    by
                } else {
                    by - 1
                };

                let x_mask = 1 << x as u32 - 1;
                let y_mask = 1 << y as u32 - 1;

                let byte_x: u32 = bytes[31 - bx as usize] as u32;
                let byte_y: u32 = bytes[31 - by as usize] as u32;
                let byte_z: u32 = bytes[31 - bz as usize] as u32;

                let oz = if z >= 8 { 0 } else { 8 - z };

                let limb = 
                    (byte_x & x_mask).wrapping_shl(y + z) + 
                    (byte_y.wrapping_shr(8 - y) & y_mask).wrapping_shl(z) + 
                    byte_z.wrapping_shr(oz);

                limbs.push(limb);
            }

            if log_limb_size == 15 {
                let limb = (bytes[0] as u32).wrapping_shr(7);
                limbs.push(limb);
                limbs.push(0u32);
            } else {
                let a: u32 = num_limbs as u32 * log_limb_size - 256;
                let limb = if (log_limb_size - a) > 8 {
                    let b: u32 = log_limb_size - a - 8;
                    (bytes[0] as u32).wrapping_shl(b) + (bytes[1] as u32).wrapping_shr(8 - b)
                } else {
                    (bytes[0] as u32).wrapping_shr(8 - (log_limb_size - a))
                };
                limbs.push(limb);
            }

            assert_eq!(limbs, expected_limbs);
        }
    }
}
