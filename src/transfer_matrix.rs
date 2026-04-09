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
pub fn calc_9x9_transfer() -> NumberOfState<82, 163> {
    poly_to_nos(&calc_transfer(9), 82)
}
pub fn calc_10x10_transfer() -> NumberOfState<101, 201> {
    poly_to_nos(&calc_transfer(10), 101)
}

fn poly_to_nos<const M: usize, const E: usize>(
    poly: &[u128],
    m_size: usize,
) -> NumberOfState<M, E> {
    let mut nos = NumberOfState::<M, E>::new();
    for e in 0..E {
        for m in 0..M {
            nos.data[e][m] = poly[e * m_size + m];
        }
    }
    nos
}

fn reverse_bits(x: usize, n: usize) -> usize {
    let mut result = 0;
    let mut x = x;
    for _ in 0..n {
        result = (result << 1) | (x & 1);
        x >>= 1;
    }
    result
}

/// Compute orbits of {0, ..., 2^n - 1} under the dihedral group D_n
/// (cyclic bit rotation + bit reversal).
/// Returns vec of (representative, orbit_size). Representative is the minimum element.
fn compute_dihedral_orbits(n: usize) -> Vec<(usize, usize)> {
    let dim = 1usize << n;
    let mask = dim - 1;
    let mut visited = vec![false; dim];
    let mut orbits = Vec::new();

    for s in 0..dim {
        if visited[s] {
            continue;
        }
        let mut size = 0;
        let mut cur = s;
        for _ in 0..n {
            if !visited[cur] {
                visited[cur] = true;
                size += 1;
            }
            cur = ((cur >> 1) | ((cur & 1) << (n - 1))) & mask;
        }
        cur = reverse_bits(s, n);
        for _ in 0..n {
            if !visited[cur] {
                visited[cur] = true;
                size += 1;
            }
            cur = ((cur >> 1) | ((cur & 1) << (n - 1))) & mask;
        }
        orbits.push((s, size));
    }

    orbits
}

/// Transfer matrix method for N x N periodic Ising model.
///
/// Uses dihedral symmetry (cyclic translation + reflection) to reduce
/// the number of independent rows from 2^N to the number of binary bracelets,
/// giving approximately N-fold speedup.
///
/// The last `fused_steps` multiplication steps are fused with the trace
/// computation in u128 to avoid u64 overflow for large N.
fn calc_transfer(n: usize) -> Vec<u128> {
    let dim = 1usize << n;
    let e_size = 2 * n * n + 1;
    let m_size = n * n + 1;
    let poly_size = e_size * m_size;
    let row_mask = (1u32 << n) - 1;

    let orbits = compute_dihedral_orbits(n);
    let num_orbits = orbits.len();

    let h_ene: Vec<usize> = (0..dim)
        .map(|j| {
            let j = j as u32;
            let shifted = ((j >> 1) | ((j & 1) << (n as u32 - 1))) & row_mask;
            (j ^ shifted).count_ones() as usize
        })
        .collect();
    let mag: Vec<usize> = (0..dim).map(|j| (j as u32).count_ones() as usize).collect();

    let by_popcount: Vec<Vec<usize>> = (0..=n)
        .map(|v| {
            (0..dim)
                .filter(|&d| (d as u32).count_ones() as usize == v)
                .collect()
        })
        .collect();

    // For large N, fuse more steps with the trace to avoid u64 overflow
    let fused_steps = if n >= 10 { 2 } else { 1 };
    let main_steps = (n - 1).saturating_sub(fused_steps);

    let mut a = vec![0u64; num_orbits * dim * poly_size];
    for (oi, &(rep, _)) in orbits.iter().enumerate() {
        for to in 0..dim {
            let de = h_ene[to] + ((rep as u32) ^ (to as u32)).count_ones() as usize;
            let dm = mag[to];
            a[(oi * dim + to) * poly_size + de * m_size + dm] = 1;
        }
    }

    let mut b = vec![0u64; num_orbits * dim * poly_size];
    let num_threads = std::thread::available_parallelism()
        .map(|p| p.get().min(MAX_THREADS).min(num_orbits))
        .unwrap_or(1);

    for step in 0..main_steps {
        let e_max = 2 * n * (step + 1);
        let m_len = n * (step + 1) + 1;

        b.fill(0);

        std::thread::scope(|scope| {
            let a = a.as_slice();
            let h_ene = &h_ene;
            let mag = &mag;
            let by_popcount = &by_popcount;
            let orbits_per_thread = num_orbits.div_ceil(num_threads);
            let mut b_rest = b.as_mut_slice();

            for tid in 0..num_threads {
                let oi_start = tid * orbits_per_thread;
                if oi_start >= num_orbits {
                    break;
                }
                let oi_end = (oi_start + orbits_per_thread).min(num_orbits);
                let num_oi = oi_end - oi_start;
                let (chunk, tail) = b_rest.split_at_mut(num_oi * dim * poly_size);
                b_rest = tail;

                scope.spawn(move || {
                    let mut temp = vec![0u64; poly_size];

                    for oi in oi_start..oi_end {
                        let li = oi - oi_start;
                        for j in 0..dim {
                            let h = h_ene[j];
                            let dm = mag[j];
                            let b_base = (li * dim + j) * poly_size;

                            for (v, group) in by_popcount.iter().enumerate() {
                                let de = h + v;

                                if group.len() == 1 {
                                    let k = j ^ group[0];
                                    let a_base = (oi * dim + k) * poly_size;
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

                                let k0 = j ^ group[0];
                                let a0 = (oi * dim + k0) * poly_size;
                                for e in 0..=e_max {
                                    temp[e * m_size..][..m_len]
                                        .copy_from_slice(&a[a0 + e * m_size..][..m_len]);
                                }
                                for &delta in &group[1..] {
                                    let k = j ^ delta;
                                    let a_base = (oi * dim + k) * poly_size;
                                    for e in 0..=e_max {
                                        let src = &a[a_base + e * m_size..][..m_len];
                                        let dst = &mut temp[e * m_size..][..m_len];
                                        for (d, &s) in dst.iter_mut().zip(src) {
                                            *d += s;
                                        }
                                    }
                                }

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

    // Free B to reclaim memory for the fused trace
    drop(b);

    // A = T^(main_steps + 1) = T^(n - fused_steps)
    let mut result = vec![0u128; poly_size];

    if fused_steps == 1 {
        // Single-step fused trace: Tr(A × T_one) in u128
        let e_max_a = 2 * n * (n - 1);
        let m_len_a = n * (n - 1) + 1;

        for (oi, &(rep, orbit_size)) in orbits.iter().enumerate() {
            let h = h_ene[rep];
            let dm = mag[rep];
            let size128 = orbit_size as u128;

            for (v, group) in by_popcount.iter().enumerate() {
                let de = h + v;
                for &delta in group.iter() {
                    let k = rep ^ delta;
                    let a_base = (oi * dim + k) * poly_size;
                    for e in 0..=e_max_a {
                        let src = &a[a_base + e * m_size..][..m_len_a];
                        let dst = &mut result[(e + de) * m_size + dm..][..m_len_a];
                        for (d, &s) in dst.iter_mut().zip(src) {
                            *d += s as u128 * size128;
                        }
                    }
                }
            }
        }
    } else {
        // Two-step fused trace: temp = A × T_one (u128), then Tr(temp × T_one) in u128
        // Parallelized: each thread processes a subset of orbits with its own temp buffer
        let e_max_a = 2 * n * (n - 2);
        let m_len_a = n * (n - 2) + 1;
        let e_max_temp = 2 * n * (n - 1);
        let m_len_temp = n * (n - 1) + 1;

        std::thread::scope(|scope| {
            let a = a.as_slice();
            let h_ene = &h_ene;
            let mag = &mag;
            let by_popcount = &by_popcount;
            let orbits = &orbits;
            let orbits_per_thread = num_orbits.div_ceil(num_threads);

            let handles: Vec<_> = (0..num_threads)
                .map(|tid| {
                    let oi_start = tid * orbits_per_thread;
                    let oi_end = (oi_start + orbits_per_thread).min(num_orbits);
                    scope.spawn(move || {
                        if oi_start >= num_orbits {
                            return vec![0u128; 0];
                        }
                        let mut partial = vec![0u128; poly_size];
                        let mut temp = vec![0u128; dim * poly_size];

                        for oi in oi_start..oi_end {
                            // Step 1: temp[l] = Σ_k A[oi][k] ⊗ T_one[k][l]
                            temp.fill(0);
                            for l in 0..dim {
                                let h = h_ene[l];
                                let dm = mag[l];
                                for (v, group) in by_popcount.iter().enumerate() {
                                    let de = h + v;
                                    for &delta in group.iter() {
                                        let k = l ^ delta;
                                        let a_base = (oi * dim + k) * poly_size;
                                        let t_base = l * poly_size;
                                        for e in 0..=e_max_a {
                                            let src = &a[a_base + e * m_size..][..m_len_a];
                                            let dst = &mut temp[t_base + (e + de) * m_size + dm..]
                                                [..m_len_a];
                                            for (d, &s) in dst.iter_mut().zip(src) {
                                                *d += s as u128;
                                            }
                                        }
                                    }
                                }
                            }

                            // Step 2: partial += |oi| × Σ_l temp[l] ⊗ T_one[l][rep]
                            let (rep, orbit_size) = orbits[oi];
                            let h = h_ene[rep];
                            let dm = mag[rep];
                            let size128 = orbit_size as u128;
                            for (v, group) in by_popcount.iter().enumerate() {
                                let de = h + v;
                                for &delta in group.iter() {
                                    let l = rep ^ delta;
                                    let t_base = l * poly_size;
                                    for e in 0..=e_max_temp {
                                        let src = &temp[t_base + e * m_size..][..m_len_temp];
                                        let dst =
                                            &mut partial[(e + de) * m_size + dm..][..m_len_temp];
                                        for (d, &s) in dst.iter_mut().zip(src) {
                                            *d += s * size128;
                                        }
                                    }
                                }
                            }
                        }
                        partial
                    })
                })
                .collect();

            for handle in handles {
                let partial = handle.join().unwrap();
                if !partial.is_empty() {
                    for (r, &p) in result.iter_mut().zip(partial.iter()) {
                        *r += p;
                    }
                }
            }
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dihedral_orbits() {
        let orbits = compute_dihedral_orbits(2);
        assert_eq!(orbits.iter().map(|o| o.1).sum::<usize>(), 4);
        assert_eq!(orbits.len(), 3);

        let orbits = compute_dihedral_orbits(7);
        assert_eq!(orbits.iter().map(|o| o.1).sum::<usize>(), 128);
        assert_eq!(orbits.len(), 18);

        let orbits = compute_dihedral_orbits(8);
        assert_eq!(orbits.iter().map(|o| o.1).sum::<usize>(), 256);
        assert_eq!(orbits.len(), 30);
    }

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

        let total: u128 = r.data.iter().flat_map(|row| row.iter()).copied().sum();
        assert_eq!(total, 1u128 << 49);

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
