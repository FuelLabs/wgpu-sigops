# `wgpu_secp256`

## Getting started

First, make sure to clone the `fuel_algos` repository:

```bash
git clone git@github.com:geometers/fuel_algos.git
```

Next, clone this repository and run its tests:

```bash
git clone git@github.com:geometers/wgpu_secp256.git &&
cd wgpu_secp256 &&
cargo test
```

## Montgomery multiplication benchmarks

```bash
cargo test mont_mul_benchmarks -- --nocapture
```
