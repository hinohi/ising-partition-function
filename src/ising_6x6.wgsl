// 6x6 Ising model: enumerate all 2^36 spin configurations,
// accumulate (energy, magnetization) histogram.
//
// Cell is 36 bits, split into lo (bits 0-31) and hi (bits 32-35).
// Dispatch: 1024 x 1024 workgroups, 256 threads each, 256 configs per thread.
// Total: 1024 * 1024 * 256 * 256 = 2^36 configurations.

@group(0) @binding(0) var<storage, read_write> hist_lo: array<atomic<u32>, 2701>;
@group(0) @binding(1) var<storage, read_write> hist_hi: array<atomic<u32>, 2701>;

var<workgroup> local_hist: array<atomic<u32>, 2701>;

const DISPATCH_X: u32 = 1024u;
const WORKGROUP_SIZE: u32 = 256u;
const CONFIGS_PER_THREAD: u32 = 256u;
const HIST_SIZE: u32 = 2701u;
const M_SIZE: u32 = 37u;

// MASK_A = 0b011111_011111_011111_011111_011111_011111
const MASK_A_LO: u32 = 0xDF7DF7DFu;
const MASK_A_HI: u32 = 0x7u;
// MASK_B = 0b100000_100000_100000_100000_100000_100000
const MASK_B_LO: u32 = 0x20820820u;
const MASK_B_HI: u32 = 0x8u;

@compute @workgroup_size(256)
fn main(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    // Initialize local histogram to zero
    for (var i = lid.x; i < HIST_SIZE; i += WORKGROUP_SIZE) {
        atomicStore(&local_hist[i], 0u);
    }
    workgroupBarrier();

    // Compute start cell for this thread
    // thread_id fits in u32 (max = 2^28 - 1)
    let thread_id = (wid.y * DISPATCH_X + wid.x) * WORKGROUP_SIZE + lid.x;
    // start = thread_id * 256; split into (lo, hi)
    // Since thread_id < 2^28, start < 2^36
    let start_lo = thread_id << 8u;
    // hi is constant across all 256 iterations (low 8 bits of start_lo are 0)
    let hi = thread_id >> 24u;

    for (var j = 0u; j < CONFIGS_PER_THREAD; j++) {
        // cell = start + j; since low 8 bits of start_lo are 0 and j < 256, no carry
        let lo = start_lo | j;

        // Magnetization: popcount of 36-bit cell
        let mag = countOneBits(lo) + countOneBits(hi & 0xFu);

        // slide_y = cell >> 6 | (cell & 0x3F) << 30
        let sy_lo = (lo >> 6u) | (hi << 26u) | ((lo & 0x3Fu) << 30u);
        let sy_hi = ((lo & 0x3Fu) >> 2u) & 0xFu;

        // slide_x = (cell >> 1) & MASK_A | (cell << 5) & MASK_B
        let sx_lo = (((lo >> 1u) | (hi << 31u)) & MASK_A_LO)
                  | ((lo << 5u) & MASK_B_LO);
        let sx_hi = ((hi >> 1u) & MASK_A_HI)
                  | (((lo >> 27u) | (hi << 5u)) & MASK_B_HI);

        // Energy: popcount of XOR with neighbors
        let ene = countOneBits(lo ^ sx_lo) + countOneBits((hi & 0xFu) ^ sx_hi)
                + countOneBits(lo ^ sy_lo) + countOneBits((hi & 0xFu) ^ sy_hi);

        let idx = ene * M_SIZE + mag;
        atomicAdd(&local_hist[idx], 1u);
    }

    workgroupBarrier();

    // Flush local histogram to global with u64 emulation (lo + hi carry)
    for (var i = lid.x; i < HIST_SIZE; i += WORKGROUP_SIZE) {
        let val = atomicLoad(&local_hist[i]);
        if (val > 0u) {
            let old = atomicAdd(&hist_lo[i], val);
            // Carry: old + val overflowed u32
            if (old > 0xFFFFFFFFu - val) {
                atomicAdd(&hist_hi[i], 1u);
            }
        }
    }
}
