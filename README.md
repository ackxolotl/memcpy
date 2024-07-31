# memcpy

How fast is `memcpy` with SIMD? Run the benchmarks:

```sh
RUSTFLAGS="-Ctarget-cpu=native" cargo bench
```

## Some results

Measured on my `AMD Ryzen 9 7950X 16-Core Processor`:

![Performance](violin.svg "Performance")