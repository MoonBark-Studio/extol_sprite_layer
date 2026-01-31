# Performance Baseline - extol_sprite_layer

**Date**: 2026-01-31
**Bevy Version**: 0.18
**Test Environment**: Release build with criterion

## Baseline Benchmark Results

These results represent the **pre-optimization** performance characteristics of `extol_sprite_layer`.

### Raw Results

| Entities | Y-Sorted Time | Unsorted Time | Throughput (Y-Sorted) | Throughput (Unsorted) |
|----------|---------------|---------------|----------------------|----------------------|
| 1,000 | 21.06 µs | 20.83 µs | 47.48 Melem/s | 48.02 Melem/s |
| 2,000 | 39.84 µs | 40.59 µs | 50.20 Melem/s | 49.28 Melem/s |
| 4,000 | 77.62 µs | 77.25 µs | 51.53 Melem/s | 51.78 Melem/s |
| 8,000 | 154.46 µs | 152.96 µs | 51.79 Melem/s | 52.30 Melem/s |
| 16,000 | 310.39 µs | 313.93 µs | 51.55 Melem/s | 50.97 Melem/s |

### Key Metrics

**Linear Scaling Coefficient**: ~19.4 µs per 1,000 entities

**Frame Budget Analysis** (at 60 FPS = 16.67ms frame budget):

| Entity Count | Time | % of Frame Budget |
|--------------|------|-------------------|
| 1,000 | 21 µs | 0.13% |
| 2,000 | 40 µs | 0.24% |
| 4,000 | 78 µs | 0.47% |
| 8,000 | 154 µs | 0.92% |
| 16,000 | 310 µs | 1.86% |

### Observations

1. **Y-sorting overhead is minimal**: Difference between y-sorted and unsorted is within measurement variance (~1-2%)
2. **Consistent throughput**: ~50-52M elements/second across all scales
3. **Linear scaling confirmed**: O(n) complexity as expected
4. **No allocation pooling**: Each frame allocates new HashMap and Vec for y-sorting

### Memory Allocations Per Frame

Based on code analysis:

| Allocation | Size (1K entities) | Size (16K entities) |
|------------|-------------------|---------------------|
| EntityHashMap<Layer> | ~32 KB | ~512 KB |
| Vec<Entity> (y-sort) | ~8 KB | ~128 KB |
| **Total per frame** | **~40 KB** | **~640 KB** |
| **Per second @ 60 FPS** | **~2.4 MB** | **~38.4 MB** |

### Identified Optimization Targets

See `PERFORMANCE_AUDIT.md` for detailed analysis. Primary targets:

1. **Resource-based HashMap pooling** - Eliminate HashMap allocation per frame
2. **Vec pooling for y-sort** - Eliminate Vec allocation per frame
3. **Incremental sorting** - Only re-sort changed entities
4. **Query iteration optimization** - Improve cache locality during sort

### Expected Improvements

| Optimization | Expected Impact | Priority |
|--------------|-----------------|----------|
| Resource pooling | 20-30% reduction | HIGH |
| Vec pooling | 15-25% reduction | HIGH |
| Incremental updates | 30-50% for static scenes | MEDIUM |
| Query optimization | 10-15% reduction | MEDIUM |

**Combined estimated improvement**: 35-55% frame time reduction for large entity counts.

### Next Steps

1. Implement resource pooling for HashMap and Vec allocations
2. Rebenchmark and compare against this baseline
3. Evaluate incremental sorting if static scene performance is critical

---

*This document will be updated with post-optimization results for comparison.*
