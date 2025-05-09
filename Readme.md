# Napkin-Benches

Benchmarks of common operations in Rust, using standard and non-standard libraries.
I run these benchmarks on my laptop (generated with `system_profiler SPHardwareDataType`, `sysctl -a | grep cache` `sysctl -a | grep perflevel`):
```
Model Name: MacBook Pro
Model Identifier: Mac16,7
Chip: Apple M4 Pro
Total Number of Cores: 14 (10 Performance cluster + 4 Efficiency cluster)
Memory: 48 GB
Cache line Size: 128 B

// Performance cluster (10 cores)
L1i Size: 192 KB
L1d Size: 128 KB
L2 Size: 16.384 MB (5 cores per L2)

// Efficiency cluster (4 cores)
L1i Size: 128 KB
L1d Size: 64 KB
L2 Size: 4.096 MB (4 cores per L2)
```
