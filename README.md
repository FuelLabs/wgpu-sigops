# `wgpu_sig_ops`

## Getting started

First, make sure to clone the `fuel_algos` repository:

```bash
git clone git@github.com:geometers/fuel-algos.git
```

Next, clone this repository and run its tests:

```bash
git clone git@github.com:geometers/wgpu_sig_ops.git &&
cd wgpu_sig_ops &&
cargo test
```

## Montgomery multiplication benchmarks

```bash
cargo test mont_mul_benchmarks -- --nocapture
```
