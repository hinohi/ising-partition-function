use crate::NumberOfState;

const MAX_THREADS: usize = 10;

pub fn calc_2x2_transfer() -> NumberOfState<5, 9> {
    poly_to_nos(&calc_transfer(2), 5)
}
pub fn calc_3x3_transfer() -> NumberOfState<10, 19> {
    poly_to_nos(&calc_transfer(3), 10)
}
pub fn calc_4x4_transfer() -> NumberOfState<17, 33> {
    poly_to_nos(&calc_transfer(4), 17)
}
pub fn calc_5x5_transfer() -> NumberOfState<26, 51> {
    poly_to_nos(&calc_transfer(5), 26)
}
pub fn calc_6x6_transfer() -> NumberOfState<37, 73> {
    poly_to_nos(&calc_transfer(6), 37)
}
pub fn calc_7x7_transfer() -> NumberOfState<50, 99> {
    poly_to_nos(&calc_transfer(7), 50)
}
pub fn calc_8x8_transfer() -> NumberOfState<65, 129> {
    poly_to_nos(&calc_transfer(8), 65)
}

fn poly_to_nos<const M: usize, const E: usize>(poly: &[u64], m_size: usize) -> NumberOfState<M, E> {
    let mut nos = NumberOfState::<M, E>::new();
    for e in 0..E {
        for m in 0..M {
            nos.data[e][m] = poly[e * m_size + m];
        }
    }
    nos
}

/// Transfer matrix method for N x N periodic Ising model.
///
/// Key optimization: group source rows by vertical energy (Hamming weight of XOR).
/// This reduces shifted adds from dim to (n+1) per target row, replacing
/// the rest with cheaper unshifted accumulation into a temp buffer.
fn calc_transfer(n: usize) -> Vec<u64> {
    let dim = 1usize << n;
    let e_size = 2 * n * n + 1;
    let m_size = n * n + 1;
    let poly_size = e_size * m_size;
    let row_mask = (1u32 << n) - 1;

    // Precompute per-row horizontal energy and magnetization
    let h_ene: Vec<usize> = (0..dim)
        .map(|j| {
            let j = j as u32;
            let shifted = ((j >> 1) | ((j & 1) << (n as u32 - 1))) & row_mask;
            (j ^ shifted).count_ones() as usize
        })
        .collect();
    let mag: Vec<usize> = (0..dim).map(|j| (j as u32).count_ones() as usize).collect();

    // Group bit-flip deltas by Hamming weight (= vertical energy).
    // For target row j, source rows with vertical energy v are { j ^ delta : delta in group[v] }.
    let by_popcount: Vec<Vec<usize>> = (0..=n)
        .map(|v| {
            (0..dim)
                .filter(|&d| (d as u32).count_ones() as usize == v)
                .collect()
        })
        .collect();

    // Initialize A = T^1
    let mut a = vec![0u64; dim * dim * poly_size];
    for from in 0..dim {
        for to in 0..dim {
            let de = h_ene[to] + ((from as u32) ^ (to as u32)).count_ones() as usize;
            let dm = mag[to];
            a[(from * dim + to) * poly_size + de * m_size + dm] = 1;
        }
    }

    let mut b = vec![0u64; dim * dim * poly_size];
    let num_threads = std::thread::available_parallelism()
        .map(|p| p.get().min(MAX_THREADS))
        .unwrap_or(1);

    for step in 0..n - 1 {
        // Step-constant bounds (no per-entry min needed)
        let e_max = 2 * n * (step + 1);
        let m_len = n * (step + 1) + 1;

        b.fill(0);

        std::thread::scope(|scope| {
            let a = a.as_slice();
            let h_ene = &h_ene;
            let mag = &mag;
            let by_popcount = &by_popcount;
            let rows_per_thread = dim.div_ceil(num_threads);
            let mut b_rest = b.as_mut_slice();

            for tid in 0..num_threads {
                let i_start = tid * rows_per_thread;
                if i_start >= dim {
                    break;
                }
                let i_end = (i_start + rows_per_thread).min(dim);
                let num_rows = i_end - i_start;
                let (chunk, tail) = b_rest.split_at_mut(num_rows * dim * poly_size);
                b_rest = tail;

                scope.spawn(move || {
                    let mut temp = vec![0u64; poly_size];

                    for i in i_start..i_end {
                        let li = i - i_start;
                        for j in 0..dim {
                            let h = h_ene[j];
                            let dm = mag[j];
                            let b_base = (li * dim + j) * poly_size;

                            for (v, group) in by_popcount.iter().enumerate() {
                                let de = h + v;

                                if group.len() == 1 {
                                    // Single element: skip temp, add directly to B
                                    let k = j ^ group[0];
                                    let a_base = (i * dim + k) * poly_size;
                                    for e in 0..=e_max {
                                        let src = &a[a_base + e * m_size..][..m_len];
                                        let dst =
                                            &mut chunk[b_base + (e + de) * m_size + dm..][..m_len];
                                        for (d, &s) in dst.iter_mut().zip(src) {
                                            *d += s;
                                        }
                                    }
                                    continue;
                                }

                                // Multi-element: accumulate into temp (copy first, add rest)
                                let k0 = j ^ group[0];
                                let a0 = (i * dim + k0) * poly_size;
                                for e in 0..=e_max {
                                    temp[e * m_size..][..m_len]
                                        .copy_from_slice(&a[a0 + e * m_size..][..m_len]);
                                }
                                for &delta in &group[1..] {
                                    let k = j ^ delta;
                                    let a_base = (i * dim + k) * poly_size;
                                    for e in 0..=e_max {
                                        let src = &a[a_base + e * m_size..][..m_len];
                                        let dst = &mut temp[e * m_size..][..m_len];
                                        for (d, &s) in dst.iter_mut().zip(src) {
                                            *d += s;
                                        }
                                    }
                                }

                                // Shifted add: B[li][j] += shift(temp, de, dm)
                                for e in 0..=e_max {
                                    let src = &temp[e * m_size..][..m_len];
                                    let dst =
                                        &mut chunk[b_base + (e + de) * m_size + dm..][..m_len];
                                    for (d, &s) in dst.iter_mut().zip(src) {
                                        *d += s;
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });

        std::mem::swap(&mut a, &mut b);
    }

    // Trace
    let mut result = vec![0u64; poly_size];
    for i in 0..dim {
        let base = (i * dim + i) * poly_size;
        for (r, &val) in result.iter_mut().zip(&a[base..]) {
            *r += val;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_brute_force_2x2() {
        assert_eq!(crate::calc_2x2(), calc_2x2_transfer());
    }

    #[test]
    fn matches_brute_force_3x3() {
        assert_eq!(crate::calc_3x3(), calc_3x3_transfer());
    }

    #[test]
    fn matches_brute_force_4x4() {
        assert_eq!(crate::calc_4x4(), calc_4x4_transfer());
    }

    #[test]
    fn matches_brute_force_5x5() {
        assert_eq!(crate::calc_5x5(), calc_5x5_transfer());
    }

    #[test]
    fn test_7x7_total_and_symmetry() {
        let r = calc_7x7_transfer();

        let total: u128 = r
            .data
            .iter()
            .flat_map(|row| row.iter())
            .map(|&x| x as u128)
            .sum();
        assert_eq!(total, 1u128 << 49);

        // Spin-flip symmetry: count(e, m) == count(e, N^2 - m)
        for e in 0..99 {
            for m in 0..25 {
                assert_eq!(
                    r.data[e][m],
                    r.data[e][49 - m],
                    "symmetry broken at e_idx={e}, m_idx={m}"
                );
            }
        }
    }

    #[test]
    fn test_7x7_known_values() {
        let r = calc_7x7_transfer();
        assert_eq!(r.data[0][0], 1);
        assert_eq!(r.data[0][49], 1);
        assert_eq!(r.data[4][1], 49);
        assert_eq!(r.data[4][48], 49);
        assert_eq!(r.data[8][2], 1078);
        assert_eq!(r.data[8][3], 294);
        assert_eq!(r.data[8][4], 49);
    }
}
