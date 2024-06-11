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
| 1024               | 109                | 270                |
| 2048               | 180                | 240                |
| 4096               | 360                | 326                |
| 8192               | 720                | 609                |
| 16384              | 1441               | 1218               |
| 32768              | 2883               | 2207               |
| 65536              | 5768               | 4208               |
| 131072             | 11532              | 5172               |

secp256k1 signature recovery benchmarks: 

| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 32                 | 154                |
| 2048               | 68                 | 131                |
| 4096               | 129                | 123                |
| 8192               | 257                | 190                |
| 16384              | 514                | 346                |
| 32768              | 1028               | 524                |
| 65536              | 2056               | 852                |
| 131072             | 4115               | 1621               |

secp256r1 signature verification benchmarks: 

| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 256                | 127                | 197                |
| 512                | 254                | 136                |
| 1024               | 510                | 156                |
| 2048               | 1022               | 136                |
| 4096               | 2044               | 189                |
| 8192               | 4088               | 279                |
| 16384              | 8235               | 481                |
| 32768              | 16351              | 805                |
| 65536              | 42038              | 1459               |
| 131072             | 65421              | 2824               |

### Montgomery multiplication benchmarks

These benchmarks can help select the best choice of limb size for different platforms.

```bash
cargo test mont_mul_benchmarks -- --nocapture
```

## Things to do

- Modify the secp256k1 and secp256r1 signature recovery shaders write the
  recovered public key, in affine form, to the result buffer as bytes
- Modify the ed25519 signature verification shader to accept the input bytes
  without having to change their endianness.
- Modify the ed25519 signature verification shader to output a 1 or 0 depending
  on whether the signature is valid or not.
