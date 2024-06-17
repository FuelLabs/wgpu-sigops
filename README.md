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
than subsequent invocations. Expect a 1-2 minute warmup period per ECDSA or
EdDSA shader, depending on your platform.

### secp256k1 and secp256r1 ECDSA signature recovery

To perform multiple secp256k1 / secp256r1 signature recovery operations in
parallel, use `ecrecover()` in either `src/secp256k1_ecdsa.rs` or
`src/secp256r1_ecdsa.rs` respectively.

The function signature of `ecrecover` is:

```rs
pub async fn ecrecover(
    signatures: Vec<Signature>,
    messages: Vec<Message>,
    log_limb_size: u32,
) -> Vec<Vec<u8>>
```

`Signature` and `Message` are from
[`fuel-crypto`](https://crates.io/crates/fuel-crypto).

The length of `signatures` and `messages` should be the same.

`log_limb_size` indicates the bitwidth of each limb in the shaders'
representation of big integers. It should be set either 13 or 14. The resulting
performance of using either value depends on the platform on which the code
will run, but our benchmarks show no significant performance difference.

The output is a `Vec` of byte-vectors which correspond to the big-integer byte
representation of the affine public key per i-th recovery.

### ed25519 EdDSA signature verification

To perform multiple ed25519 signature verification operations in
parallel, use `ecverify()` in `src/ed25519_eddsa.rs`.

```rs
pub async fn ecverify(
    signatures: Vec<Signature>,
    messages: Vec<Message>,
    verifying_keys: Vec<VerifyingKey>,
    log_limb_size: u32,
) -> Vec<bool>
```

`Signature` is from [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek).
`Message` and `VerifyingKey` are from
`fuel-crypto`](https://crates.io/crates/fuel-crypto).

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
- curve25519 EdDSA signature verification

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
- Shamir-Strauss multiplication
- Double-and-add multiplication
- SHA512

These tests execute the same operations in CPU and in GPU, and compare the
result to ensure correctness. For instance, for the ECDSA tests, the output of
the shader is checked against the output of the relevant ECDSA signature
recovery function from the
[`fuel-crypto`](https://crates.io/crates/fuel-crypto) library.

Of particular note is that the curve25519 EdDSA signature verification shader follows the
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
graphics card (2560 cores). The CPU benchmarks were run with the `--release`
flag, and the GPU timings include data transfer both ways.

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

Further optimisations may improve GPU performance, such as precomputation
and/or the GLV method for scalar multiplication.

secp256k1 signature recovery benchmarks: 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms)
| ------------------ | ------------------ | ------------------
| 1024               | 32                 | 289                |
| 2048               | 63                 | 173                |
| 4096               | 127                | 170                |
| 8192               | 254                | 232                |
| 16384              | 508                | 347                |
| 32768              | 1016               | 570                |
| 65536              | 2033               | 926                |
| 131072             | 4132               | 1706               |

GPU timings include data transfer.

secp256r1 signature verification benchmarks: 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 256                | 124                | 214                |
| 512                | 250                | 247                |
| 1024               | 499                | 171                |
| 2048               | 999                | 185                |
| 4096               | 1999               | 192                |
| 8192               | 3998               | 321                |
| 16384              | 8001               | 447                |
| 32768              | 16007              | 650                |
| 65536              | 32016              | 1017               |
| 131072             | 63999              | 1866               |

GPU timings include data transfer.

ed25519 signature verification benchmarks: 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 98                 | 238                |
| 2048               | 196                | 247                |
| 4096               | 391                | 343                |
| 8192               | 907                | 657                |
| 16384              | 1813               | 1269               |
| 32768              | 3135               | 2394               |
| 65536              | 6271               | 5119               |
| 131072             | 12543              | 5112               |

GPU timings include data transfer.


### Montgomery multiplication benchmarks

These benchmarks can help select the best choice of limb size for different platforms.

```bash
cargo test mont_mul_benchmarks -- --nocapture
```
