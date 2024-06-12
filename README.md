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

## Overview

This repository contains GPU shaders for the following cryptographic operations:

- secp256k1 ECDSA signature recovery
- secp256r1 ECDSA signature recovery
- curve25519 EdDSA signature verification

These shaders are written to mirror the same underlying algorithms and code that Fuel nodes use.

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

- [`secp256k1`](https://crates.io/crates/secp256k1), which uses C bindings to `libsecp256k1`
- [`p256`](https://crates.io/crates/p256), a pure Rust implementation of the secp256r1 curve
- [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek), a pure Rust
  implementation of curve25519 and the ed25519 signature scheme.

Further optimisations may improve GPU performance, such as precomputation
and/or the GLV method for scalar multiplication.

ed25519 signature verification benchmarks: 

| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 92                 | 278                |
| 2048               | 221                | 242                |
| 4096               | 370                | 326                |
| 8192               | 740                | 597                |
| 16384              | 1478               | 1171               |
| 32768              | 2960               | 2159               |
| 65536              | 5922               | 4141               |
| 131072             | 11846              | 5137               |

secp256k1 signature recovery benchmarks: 

| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 32                 | 182                |
| 2048               | 64                 | 139                |
| 4096               | 128                | 138                |
| 8192               | 256                | 204                |
| 16384              | 512                | 340                |
| 32768              | 1025               | 537                |
| 65536              | 2052               | 882                |
| 131072             | 4100               | 1683               |

secp256r1 signature verification benchmarks: 

| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 256                | 126                | 188                |
| 512                | 252                | 137                |
| 1024               | 506                | 135                |
| 2048               | 1044               | 136                |
| 4096               | 2025               | 196                |
| 8192               | 4051               | 293                |
| 16384              | 8101               | 496                |
| 32768              | 16197              | 803                |
| 65536              | 32375              | 1464               |
| 131072             | 64834              | 2899               |


### Montgomery multiplication benchmarks

These benchmarks can help select the best choice of limb size for different platforms.

```bash
cargo test mont_mul_benchmarks -- --nocapture
```

## Things to do

- Modify the secp256k1 and secp256r1 signature recovery shaders write the
  recovered public key, in affine form, to the result buffer as bytes
- Modify the ed25519 signature verification shader to output a 1 or 0 depending
  on whether the signature is valid or not.
