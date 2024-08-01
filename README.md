# memcpy

How fast is `memcpy` with SIMD? Run the benchmarks:

```sh
RUSTFLAGS="-Ctarget-cpu=native" cargo bench
```

## Results

### AMD Ryzen 9 7950X 16-Core Processor

with 2x 16 GiB Kingston KF552C40-16 (DDR5-5200)

![Performance](results/amd-ryzen-9-7950x-ddr5-32gib-kingston.svg "Performance")

### Apple M1

with 16 GiB (LPDDR4X-4266)

![Performance](results/apple-m1-lpddr4x-16gib.svg "Performance")

## May I add my results?

Yes, please!
