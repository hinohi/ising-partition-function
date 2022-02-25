# Explicitly calculation of Ising Partition Function

Explicitly calculate the partition function in a small Ising model.

## Hamiltonian

```math
H = -J \sum _{\left\langle i,j \right\rangle} \sigma_i \sigma_j -h \sum_{i} \sigma_i
```

### 4x4

```
  \m  -16  -14  -12  -10   -8   -6   -4   -2    0    2    4    6    8   10   12   14   16
 e \-------------------------------------------------------------------------------------
-32|    1    0    0    0    0    0    0    0    0    0    0    0    0    0    0    0    1
-28|    0    0    0    0    0    0    0    0    0    0    0    0    0    0    0    0    0
-24|    0   16    0    0    0    0    0    0    0    0    0    0    0    0    0   16    0
-20|    0    0   32    0    0    0    0    0    0    0    0    0    0    0   32    0    0
-16|    0    0   88   96   24    0    0    0    8    0    0    0   24   96   88    0    0
-12|    0    0    0  256  256  192   96   64    0   64   96  192  256  256    0    0    0
 -8|    0    0    0  208  736  688  704  624  768  624  704  688  736  208    0    0    0
 -4|    0    0    0    0  576 1664 1824 1920 1600 1920 1824 1664  576    0    0    0    0
  0|    0    0    0    0  228 1248 2928 3680 4356 3680 2928 1248  228    0    0    0    0
  4|    0    0    0    0    0  448 1568 3136 3264 3136 1568  448    0    0    0    0    0
  8|    0    0    0    0    0  128  768 1392 2112 1392  768  128    0    0    0    0    0
 12|    0    0    0    0    0    0   64  512  576  512   64    0    0    0    0    0    0
 16|    0    0    0    0    0    0   56   96  120   96   56    0    0    0    0    0    0
 20|    0    0    0    0    0    0    0    0   64    0    0    0    0    0    0    0    0
 24|    0    0    0    0    0    0    0   16    0   16    0    0    0    0    0    0    0
 28|    0    0    0    0    0    0    0    0    0    0    0    0    0    0    0    0    0
 32|    0    0    0    0    0    0    0    0    2    0    0    0    0    0    0    0    0
```