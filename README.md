# `wgpu_sig_ops`

## Getting started

Clone this repository:

```bash
git clone git@github.com:geometers/wgpu_sig_ops.git
```

Run the tests:

```
cd wgpu_sig_ops &&
cargo test -- --skip benchmarks
```

## Usage

### Warmup

Note that the first invocation of any GPU shader will take signficantly longer
than subsequent invocations. 

| Shader | Linux + Nvidia A1000 (seconds) | Apple M1 Mini (seconds) |
|-|-|-|
| secp256k1 ECDSA (single shader)    | 129  | N/A |
| secp256r1 ECDSA (single shader)    | 125  | N/A |
| ed25519 EdDSA (single shader)      | 30   | N/A |
| secp256k1 ECDSA (multiple shaders) | 30   | 2.5 |
| secp256r1 ECDSA (multiple shaders) | 77   | 2   |
| ed25519 EdDSA (multiple shaders)   | 28   | 2.8 |

**We recommend using the multi-shader approach** because it enjoys a shorter
initial warmup time and has overall better performance compared to the
single-shader approach.

### secp256k1 and secp256r1 ECDSA signature recovery

To perform multiple secp256k1 / secp256r1 signature recovery operations in
parallel, use `ecrecover()` in either `src/secp256k1_ecdsa.rs` or
`src/secp256r1_ecdsa.rs` respectively.

This function uses the multiple-shader approach. To use the single-shader
approach, use `ecrecover_single` instead.

The function signature of `ecrecover` or `ecrecover_single` is:

```rs
pub async fn ecrecover(
    signatures: Vec<Signature>,
    messages: Vec<Message>,
    table_limbs: &Vec<u32>,
    log_limb_size: u32,
) -> Vec<Vec<u8>>
```

`Signature` and `Message` are from
[`fuel-crypto`](https://crates.io/crates/fuel-crypto).

The length of `signatures` and `messages` should be the same.

`table_limbs` are precomputed multiples of the secp256k1 or secp256r1 generator
point, and can be easily generated using `precompute::secp256k1_bases` or
`precompute::secp256r1_bases` respectively.

`log_limb_size` indicates the bitwidth of each limb in the shaders'
representation of big integers. A safe default is 13.

The output is a `Vec` of byte-vectors which correspond to the big-integer byte
representation of the affine public key per i-th recovery.

### ed25519 EdDSA signature verification

To perform multiple ed25519 signature verification operations in
parallel, use `ecverify()` in `src/ed25519_eddsa.rs`.

This function uses the multiple-shader approach. To use the single-shader
approach, use `ecverify_single` instead.

The function signature of `ecverify` or `ecverify_single` is:

```rs
pub async fn ecverify(
    signatures: Vec<Signature>,
    messages: Vec<Message>,
    verifying_keys: Vec<VerifyingKey>,
    table_limbs: &Vec<u32>,
    log_limb_size: u32,
) -> Vec<bool>
```

`Signature` is from [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek).
`Message` and `VerifyingKey` are from
[`fuel-crypto`](https://crates.io/crates/fuel-crypto).

`table_limbs` are precomputed multiples of the curve25519 generator
point, and can be easily generated using `precompute::ed25519_bases`.

The output is a `Vec` of booleans which correspond to `true` if the i-th
recovery is valid, and `false` otherwise.

### Examples

See the following source files for examples on how to invoke the GPU shaders:

- `src/benchmarks/secp256k1_ecdsa.rs`
- `src/benchmarks/secp256r1_ecdsa.rs`
- `src/benchmarks/ed25519_eddsa.rs`

## Overview

This repository contains GPU shaders for the following cryptographic operations:

- secp256k1 ECDSA signature recovery
- secp256r1 ECDSA signature recovery
- ed25519 EdDSA signature verification

These shaders are written to mirror the same underlying algorithms and code
that Fuel nodes use.

These GPU shaders are written in the [WebGPU Shader
Language](https://www.w3.org/TR/WGSL/), and are executed by the
[wgpu](https://github.com/gfx-rs/wgpu) API which works with Rust.

The constituent algorithms for these operations, which also come with their own
unit tests, include:

- Big integer addition, subtraction, multiplication, and halving
- Bytestring-to-big-integer conversion
- Finite field addition, subtraction, inversion, and multiplication
- Multiplication of finite field elements in Montgomery form
- Barrett reduction
- Square root calculation where the modulus is 3 mod 4
- Projective curve point addition and doubling
- Extended Twisted Edwards curve point addition and doubling
- Shamir-Strauss EC multiplication
- Double-and-add EC multiplication
- Fixed-base windowed EC multiplication
- SHA512

These tests execute the same operations in CPU and in GPU, and compare the
result to ensure correctness. For instance, for the ECDSA tests, the output of
the shader is checked against the output of the relevant ECDSA signature
recovery function from the
[`fuel-crypto`](https://crates.io/crates/fuel-crypto) library.

Of particular note is that the ed25519 EdDSA signature verification shader follows the
[`ed25519-dalek`](https://crates.io/crates/ed25519-dalek) implementation of
EdDSA. This is important because [not all EdDSA implementations are the
same](https://hdevalence.ca/blog/2020-10-04-its-25519am), and nodes must run
the same implementation in order to maintain consensus.

## Benchmarks

```
cargo test --release multiple_benchmarks -- --nocapture
```

### Results

The following benchmarks were run on a 13th Gen Intel(R) Core(TM) i7-13700HX
machine with an [Nvidia RTX
A1000](https://www.notebookcheck.net/NVIDIA-RTX-A1000-Laptop-GPU-GPU-Benchmarks-and-Specs.615862.0.html)
graphics card (2560 cores), a Macbook Pro (M3), and a M1 Mini. The CPU benchmarks
were run with the `--release` flag, and the GPU timings include data transfer
both ways.

For each benchmark, each signature is handled in parallel by the GPU, while the
CPU handles it serially. The results show that after a certain number of
signatures, GPU performance beats CPU.

To ensure a fair comparision, the CPU benchmarks use the same libraries that
[`fuel-crypto`](https://crates.io/crates/fuel-crypto) uses under the hood:

- [`secp256k1`](https://crates.io/crates/secp256k1), which uses C bindings to
  `libsecp256k1`
- [`p256`](https://crates.io/crates/p256), a pure Rust implementation of the
  secp256r1 curve
- [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek), a pure Rust
  implementation of curve25519 and the ed25519 signature scheme.

The results may be viewed here: https://hackmd.io/@weijiek/r1u2Ka5OR

### Montgomery multiplication benchmarks

These benchmarks can help select the best choice of limb size for different platforms.

```bash
cargo test mont_mul_benchmarks -- --nocapture
```

## Troubleshooting

### If shaders aren't cached

The second and subsequent runs of any shader should always be much faster than
the first run, because the GPU backend will compile and cache it. In some
cases, this may not be the case, leading to consistently slow runs for no
apparent reason. To address the issue, consider deleting the shader cache on
your system.

On Linux machines with Nvidia GPUs, the cache may be located at
`~/.cache/nvidia/GLCache/`.
