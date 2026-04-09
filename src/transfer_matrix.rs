use crate::NumberOfState;

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
/// Instead of enumerating all 2^(N^2) configurations, process the lattice
/// row-by-row. The transfer matrix T is 2^N x 2^N, where T[sigma, sigma']
/// encodes the energy and magnetization contribution of adding row sigma'
/// given the previous row sigma. The density of states is obtained from Tr(T^N).
///
/// Complexity: O(N * 4^N * N^2) vs O(2^(N^2)) for brute force.
fn calc_transfer(n: usize) -> Vec<u64> {
    let dim = 1usize << n;
    let e_size = 2 * n * n + 1;
    let m_size = n * n + 1;
    let poly_size = e_size * m_size;
    let row_mask = (1u32 << n) - 1;

    // Build transfer matrix: for each (from, to) row pair, compute
    // (energy_contribution, magnetization_contribution)
    let t: Vec<(usize, usize)> = (0..dim)
        .flat_map(|from| {
            (0..dim).map(move |to| {
                let to_u32 = to as u32;
                // Horizontal energy: shift right by 1 with periodic wraparound
                let shifted = ((to_u32 >> 1) | ((to_u32 & 1) << (n as u32 - 1))) & row_mask;
                let h_ene = (to_u32 ^ shifted).count_ones() as usize;
                // Vertical energy: mismatches between adjacent rows
                let v_ene = ((from as u32) ^ to_u32).count_ones() as usize;
                let mag = to_u32.count_ones() as usize;
                (h_ene + v_ene, mag)
            })
        })
        .collect();

    // Initialize A = T^1 (each entry is a single monomial)
    let mut a = vec![0u64; dim * dim * poly_size];
    for from in 0..dim {
        for to in 0..dim {
            let idx = from * dim + to;
            let (de, dm) = t[idx];
            a[idx * poly_size + de * m_size + dm] = 1;
        }
    }

    let num_threads = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1);

    // Compute T^n by repeated right-multiplication: A <- A * T
    // At step s, A = T^(s+1), and we compute T^(s+2).
    for step in 0..n - 1 {
        // Bounds on non-zero polynomial entries in current A
        let src_e_max = 2 * n * (step + 1);
        let src_m_max = n * (step + 1);

        let mut b = vec![0u64; dim * dim * poly_size];

        std::thread::scope(|scope| {
            let a = a.as_slice();
            let t = t.as_slice();
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
                    for i in i_start..i_end {
                        let li = i - i_start;
                        for k in 0..dim {
                            let a_base = (i * dim + k) * poly_size;
                            for j in 0..dim {
                                let (de, dm) = t[k * dim + j];
                                let b_base = (li * dim + j) * poly_size;
                                let e_end = if de < e_size {
                                    src_e_max.min(e_size - 1 - de)
                                } else {
                                    continue;
                                };
                                let m_len = (src_m_max + 1).min(m_size - dm);
                                for e in 0..=e_end {
                                    let src = &a[a_base + e * m_size..][..m_len];
                                    let dst =
                                        &mut chunk[b_base + (e + de) * m_size + dm..][..m_len];
                                    for (d, s) in dst.iter_mut().zip(src) {
                                        *d += s;
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });

        a = b;
    }

    // Trace: sum diagonal entries to get the partition polynomial
    let mut result = vec![0u64; poly_size];
    for i in 0..dim {
        let base = (i * dim + i) * poly_size;
        for (r, val) in result.iter_mut().zip(&a[base..base + poly_size]) {
            *r += val;
        }
    }

    result
}
