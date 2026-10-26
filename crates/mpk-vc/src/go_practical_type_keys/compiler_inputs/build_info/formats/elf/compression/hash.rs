// Copyright 2023 The Go Authors. All rights reserved.
// BSD license: ../../../../GO-BUILDINFO-LICENSE; see the repository NOTICE.
//! Streaming seed-zero xxHash64, adapted from pinned internal/zstd/xxhash.go.
const P: [u64; 5] = [
    0x9e3779b185ebca87,
    0xc2b2ae3d27d4eb4f,
    0x165667b19e3779f9,
    0x85ebca77c2b2ae63,
    0x27d4eb2f165667c5,
];
pub(super) struct Hash {
    len: u64,
    v: [u64; 4],
    buf: [u8; 32],
    used: usize,
}
fn round(v: u64, n: u64) -> u64 {
    v.wrapping_add(n.wrapping_mul(P[1]))
        .rotate_left(31)
        .wrapping_mul(P[0])
}
fn word(b: &[u8]) -> u64 {
    u64::from_le_bytes(b[..8].try_into().unwrap())
}
impl Hash {
    pub(super) fn new() -> Self {
        Self {
            len: 0,
            v: [P[0].wrapping_add(P[1]), P[1], 0, P[0].wrapping_neg()],
            buf: [0; 32],
            used: 0,
        }
    }
    fn stripe(&mut self, b: &[u8]) {
        for (v, part) in self.v.iter_mut().zip(b.chunks_exact(8)) {
            *v = round(*v, word(part));
        }
    }
    pub(super) fn update(&mut self, mut b: &[u8]) {
        self.len = self.len.wrapping_add(b.len() as u64);
        if self.used + b.len() < 32 {
            self.buf[self.used..self.used + b.len()].copy_from_slice(b);
            self.used += b.len();
            return;
        }
        if self.used != 0 {
            let n = 32 - self.used;
            self.buf[self.used..].copy_from_slice(&b[..n]);
            b = &b[n..];
            let stripe = self.buf;
            self.stripe(&stripe);
            self.used = 0;
        }
        while b.len() >= 32 {
            self.stripe(&b[..32]);
            b = &b[32..];
        }
        self.buf[..b.len()].copy_from_slice(b);
        self.used = b.len();
    }
    pub(super) fn digest(&self) -> u64 {
        let mut h = if self.len < 32 {
            P[4]
        } else {
            self.v[0]
                .rotate_left(1)
                .wrapping_add(self.v[1].rotate_left(7))
                .wrapping_add(self.v[2].rotate_left(12))
                .wrapping_add(self.v[3].rotate_left(18))
        };
        if self.len >= 32 {
            for v in self.v {
                h = (h ^ round(0, v)).wrapping_mul(P[0]).wrapping_add(P[3]);
            }
        }
        h = h.wrapping_add(self.len);
        let mut b = &self.buf[..self.used];
        while b.len() >= 8 {
            h = (h ^ round(0, word(b)))
                .rotate_left(27)
                .wrapping_mul(P[0])
                .wrapping_add(P[3]);
            b = &b[8..];
        }
        if b.len() >= 4 {
            h = (h ^ u64::from(u32::from_le_bytes(b[..4].try_into().unwrap())).wrapping_mul(P[0]))
                .rotate_left(23)
                .wrapping_mul(P[1])
                .wrapping_add(P[2]);
            b = &b[4..];
        }
        for &v in b {
            h = (h ^ u64::from(v).wrapping_mul(P[4]))
                .rotate_left(11)
                .wrapping_mul(P[0]);
        }
        h ^= h >> 33;
        h = h.wrapping_mul(P[1]);
        h ^= h >> 29;
        h = h.wrapping_mul(P[2]);
        h ^ (h >> 32)
    }
}
