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
than subsequent invocations. Expect a 2-4 minute warmup period in total,
depending on your platform.

| Shader | Linux + Nvidia A1000 (seconds) | Macbook Pro (M2) (seconds) |
|-|-|-|
| secp256k1 ECDSA (single shader)    | 56  | TBC |
| secp256r1 ECDSA (single shader)    | 121 | TBC |
| ed25519 EdDSA (single shader)      | 52  | TBC |
| secp256k1 ECDSA (multiple shaders) | 30  | TBC |
| secp256r1 ECDSA (multiple shaders) | 77  | TBC |
| ed25519 EdDSA (multiple shaders)   | 56  | TBC |

See below for the a detaile discussion about the differences between the
single-shader and multiple-shader approaches.

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

This function uses the multiple-shader approach. To use the single-shader
approach, use `ecverify_single` instead.

The function signature of `ecverify` or `ecverify_single` is:

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
[`fuel-crypto`](https://crates.io/crates/fuel-crypto).

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


<!--
#### Summary

We found that the Nvidia A1000 GPU on a Linux machine performed consistently
faster than the Macbook Pro (M2) on the multiple-shader approach.

With the single-shader approach, Nvidia A1000 GPU performed about the same as
it did with the multiple-shader approach.

Unfortunately, the Macbook Pro (M1 and M2) did not work with the single-shader
approach at all.

Finally, the warmup period for the Nvidia A1000 was signficantly faster for the
multiple-shader approach compared to the single-shader approach.

To attempt to make the Macbook Pro a viable platform for executing these
shaders in production, we will implement the following two optimisations:

- Precomputed lookup tables for scalar multiplication of the point generator
- The GLV method for variable-base scalar multiplication
-->

#### Multiple-shader benchmarks

To get the GPU shaders working on Macbook Pros with the M2 chip, it was
necessary for us to implement the GPU code as multiple shaders. Otherwise, we
ran into execution errors.

Each shader would perform part of the computation while keeping the output in
GPU memory. The final result would only be read from GPU memory once the
sequence of shader execution is complete.

##### Linux + Nvidia A1000

secp256k1 signature recovery benchmarks (multiple shaders): 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 32                 | 172                |
| 2048               | 63                 | 156                |
| 4096               | 127                | 158                |
| 8192               | 254                | 219                |
| 16384              | 509                | 338                |
| 32768              | 1018               | 554                |
| 65536              | 2033               | 933                |
| 131072             | 4066               | 1695               |

GPU timings include data transfer.

secp256r1 signature verification benchmarks (multiple shaders): 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 256                | 125                | 374                |
| 512                | 250                | 168                |
| 1024               | 500                | 196                |
| 2048               | 1001               | 213                |
| 4096               | 2007               | 194                |
| 8192               | 4014               | 334                |
| 16384              | 8015               | 449                |
| 32768              | 16063              | 639                |
| 65536              | 32088              | 1027               |
| 131072             | 64125              | 1861               |

GPU timings include data transfer.

ed25519 signature verification benchmarks (multiple shaders): 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 92                 | 333                |
| 2048               | 181                | 328                |
| 4096               | 363                | 413                |
| 8192               | 955                | 8848               |
| 16384              | 1752               | 3344               |
| 32768              | 3506               | 2572               |
| 65536              | 5818               | 4780               |
| 131072             | 11623              | 10346              |

GPU timings include data transfer.

##### Macbook Pro (M2)

secp256k1 signature recovery benchmarks: 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 30                 | 918                |
| 2048               | 55                 | 939                |
| 4096               | 110                | 1879               |
| 8192               | 221                | 3632               |
| 16384              | 453                | 5885               |
| 32768              | 909                | 10021              |
| 65536              | 1826               | 10027              |
| 131072             | 3611               | 10045              |

secp256r1 signature verification benchmarks (multiple shaders): 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 256                | 115                | 1007               |
| 512                | 230                | 1061               |
| 1024               | 461                | 1143               |
| 2048               | 923                | 1238               |
| 4096               | 1842               | 2375               |
| 8192               | 3686               | 4176               |
| 16384              | 7390               | 8575               |
| 32768              | 15165              | 10023              |
| 65536              | 30358              | 10023              |
| 131072             | 60801              | 10034              |

GPU timings include data transfer.

ed25519 signature verification benchmarks (multiple shaders): 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 88                 | 7408               |
| 2048               | 178                | 651                |
| 4096               | 360                | 688                |
| 8192               | 711                | 2049               |
| 16384              | 1508               | 2330               |
| 32768              | 2938               | 5931               |
| 65536              | 5848               | 10032              |
| 131072             | 11809              | 10035              |

#### Single-shader benchmarks

On the Linux machine with an Nvidia A1000 GPU, we found that performing the
whole computation using a single shader had a very slight performance
advantage over splitting up the computation into multiple shaders.

##### Linux + Nvidia A1000

secp256k1 signature recovery benchmarks (single shader): 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 32                 | 257                |
| 2048               | 64                 | 158                |
| 4096               | 127                | 150                |
| 8192               | 261                | 215                |
| 16384              | 508                | 331                |
| 32768              | 1018               | 549                |
| 65536              | 2032               | 913                |
| 131072             | 4065               | 1718               |

secp256r1 signature verification benchmarks (single shader): 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 256                | 126                | 164                |
| 512                | 256                | 151                |
| 1024               | 500                | 163                |
| 2048               | 1002               | 179                |
| 4096               | 2007               | 215                |
| 8192               | 4005               | 360                |
| 16384              | 8028               | 545                |
| 32768              | 16059              | 898                |
| 65536              | 32126              | 1542               |
| 131072             | 64223              | 2926               |

ed25519 signature verification benchmarks (single shader): 
| Num. signatures    | CPU, serial (ms)   | GPU, parallel (ms) |
| ------------------ | ------------------ | ------------------ |
| 1024               | 109                | 376                |
| 2048               | 181                | 276                |
| 4096               | 449                | 364                |
| 8192               | 726                | 657                |
| 16384              | 1453               | 1253               |
| 32768              | 3599               | 2379               |
| 65536              | 5999               | 4400               |
| 131072             | 11627              | 10663              |

##### Macbook Pro (M2)

Failed to run (`Compute function exceeds available stack space`).

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
