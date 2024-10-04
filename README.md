# memcpy

How fast can we `memcpy` 256 MiB of data? Run the benchmarks:

```sh
RUSTFLAGS="-Ctarget-cpu=native" cargo bench
```

## Results

### Intel(R) Xeon(R) CPU E5-2660 v2

DDR3-1866

![Performance](results/intel-xeon-e5-2660-v2-ddr3-256gib-samsung.svg "Performance")

### AMD EPYC 7713 64-Core Processor

DDR4-3200

![Performance](results/amd-epyc-7713-ddr4-1024gib-samsung.svg "Performance")

### AMD EPYC 9454P 48-Core Processor

DDR5-4800

![Performance](results/amd-epyc-9454p-ddr5-384gib-micron.svg "Performance")

### AMD Ryzen 7 1800X Eight-Core Processor

DDR4-2133

![Performance](results/amd-ryzen-7-1800x-ddr4-32gib-gskill.svg "Performance")

### AMD Ryzen 9 7950X 16-Core Processor

DDR5-5200

![Performance](results/amd-ryzen-9-7950x-ddr5-32gib-kingston.svg "Performance")

### Apple M1

LPDDR4X-4266

![Performance](results/apple-m1-lpddr4x-16gib.svg "Performance")

### Apple M1 Pro

LPDDR5-6400

![Performance](results/apple-m1-pro-lpddr5-16gib.svg "Performance")

### Apple M2

LPDDR5-6400

![Performance](results/apple-m2-lpddr5-16gib.svg "Performance")

### Apple M2 Max

LPDDR5-6400

![Performance](results/apple-m2-max-lpddr5-32gib.svg "Performance")

### Apple M3 Pro

LPDDR5-6400

![Performance](results/apple-m3-pro-lpddr5-36gib.svg "Performance")

## May I add my results?

Yes, please!
