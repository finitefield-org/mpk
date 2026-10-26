// Copyright 2022 The Go Authors. All rights reserved.
// Derived from Go 1.25 sort/zsortfunc.go and sort/sort.go under the BSD
// license in GO-SORT-LICENSE. Only the unstable sorting algorithm is ported.
// Equal semantic version labels must follow the pinned Go sort permutation:
// neither Rust's sort nor an inventory byte tiebreak specifies that order.
use super::*;
fn less(d: &[GoSourceGraphModule], a: usize, b: usize) -> bool {
    d[a].path
        .cmp(&d[b].path)
        .then_with(|| d[a].precedence(&d[b]))
        .is_lt()
}
pub(super) fn sort(d: &mut [GoSourceGraphModule]) {
    let n = d.len();
    let limit = usize::BITS - n.leading_zeros();
    pdq(d, 0, n, limit);
}
fn insertion(d: &mut [GoSourceGraphModule], a: usize, b: usize) {
    for i in a + 1..b {
        let mut j = i;
        while j > a && less(d, j, j - 1) {
            d.swap(j, j - 1);
            j -= 1;
        }
    }
}
fn sift(d: &mut [GoSourceGraphModule], mut root: usize, hi: usize, first: usize) {
    loop {
        let mut child = 2 * root + 1;
        if child >= hi {
            break;
        }
        if child + 1 < hi && less(d, first + child, first + child + 1) {
            child += 1;
        }
        if !less(d, first + root, first + child) {
            return;
        }
        d.swap(first + root, first + child);
        root = child;
    }
}
fn heap(d: &mut [GoSourceGraphModule], a: usize, b: usize) {
    let hi = b - a;
    for i in (0..=(hi - 1) / 2).rev() {
        sift(d, i, hi, a);
    }
    for i in (0..hi).rev() {
        d.swap(a, a + i);
        sift(d, 0, i, a);
    }
}
fn pdq(d: &mut [GoSourceGraphModule], mut a: usize, mut b: usize, mut limit: u32) {
    let mut balanced = true;
    let mut partitioned = true;
    loop {
        let len = b - a;
        if len <= 12 {
            insertion(d, a, b);
            return;
        }
        if limit == 0 {
            heap(d, a, b);
            return;
        }
        if !balanced {
            break_patterns(d, a, b);
            limit -= 1;
        }
        let (mut pivot, mut hint) = choose(d, a, b);
        if hint == -1 {
            d[a..b].reverse();
            pivot = (b - 1) - (pivot - a);
            hint = 1;
        }
        if balanced && partitioned && hint == 1 && partial(d, a, b) {
            return;
        }
        if a > 0 && !less(d, a - 1, pivot) {
            a = equal(d, a, b, pivot);
            continue;
        }
        let (mid, done) = partition(d, a, b, pivot);
        partitioned = done;
        let (left, right) = (mid - a, b - mid);
        let threshold = len / 8;
        if left < right {
            balanced = left >= threshold;
            pdq(d, a, mid, limit);
            a = mid + 1;
        } else {
            balanced = right >= threshold;
            pdq(d, mid + 1, b, limit);
            b = mid;
        }
    }
}
fn partition(d: &mut [GoSourceGraphModule], a: usize, b: usize, pivot: usize) -> (usize, bool) {
    d.swap(a, pivot);
    let (mut i, mut j) = (a + 1, b - 1);
    while i <= j && less(d, i, a) {
        i += 1;
    }
    while i <= j && !less(d, j, a) {
        j -= 1;
    }
    if i > j {
        d.swap(j, a);
        return (j, true);
    }
    d.swap(i, j);
    i += 1;
    j -= 1;
    loop {
        while i <= j && less(d, i, a) {
            i += 1;
        }
        while i <= j && !less(d, j, a) {
            j -= 1;
        }
        if i > j {
            break;
        }
        d.swap(i, j);
        i += 1;
        j -= 1;
    }
    d.swap(j, a);
    (j, false)
}
fn equal(d: &mut [GoSourceGraphModule], a: usize, b: usize, pivot: usize) -> usize {
    d.swap(a, pivot);
    let (mut i, mut j) = (a + 1, b - 1);
    loop {
        while i <= j && !less(d, a, i) {
            i += 1;
        }
        while i <= j && less(d, a, j) {
            j -= 1;
        }
        if i > j {
            break;
        }
        d.swap(i, j);
        i += 1;
        j -= 1;
    }
    i
}
fn partial(d: &mut [GoSourceGraphModule], a: usize, b: usize) -> bool {
    let mut i = a + 1;
    for _ in 0..5 {
        while i < b && !less(d, i, i - 1) {
            i += 1;
        }
        if i == b {
            return true;
        }
        if b - a < 50 {
            return false;
        }
        d.swap(i, i - 1);
        if i - a >= 2 {
            let mut j = i - 1;
            while j >= 1 && less(d, j, j - 1) {
                d.swap(j, j - 1);
                j -= 1;
            }
        }
        if b - i >= 2 {
            for j in i + 1..b {
                if !less(d, j, j - 1) {
                    break;
                }
                d.swap(j, j - 1);
            }
        }
    }
    false
}
fn break_patterns(d: &mut [GoSourceGraphModule], a: usize, b: usize) {
    let len = b - a;
    if len >= 8 {
        let mut random = len as u64;
        // Go nextPowerOfTwo uses the next power strictly greater than len.
        let mask = usize::MAX >> len.leading_zeros();
        for idx in a + (len / 4) * 2 - 1..=a + (len / 4) * 2 + 1 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            let mut other = (random as usize) & mask;
            if other >= len {
                other -= len;
            }
            d.swap(idx, a + other);
        }
    }
}
fn order(d: &[GoSourceGraphModule], a: usize, b: usize, swaps: &mut u32) -> (usize, usize) {
    if less(d, b, a) {
        *swaps += 1;
        (b, a)
    } else {
        (a, b)
    }
}
fn median(d: &[GoSourceGraphModule], a: usize, b: usize, c: usize, swaps: &mut u32) -> usize {
    let (a, b) = order(d, a, b, swaps);
    let (b, _) = order(d, b, c, swaps);
    let (_, b) = order(d, a, b, swaps);
    b
}
fn choose(d: &[GoSourceGraphModule], a: usize, b: usize) -> (usize, i8) {
    let len = b - a;
    let (mut i, mut j, mut k) = (a + len / 4, a + len / 4 * 2, a + len / 4 * 3);
    let mut swaps = 0;
    if len >= 8 {
        if len >= 50 {
            i = median(d, i - 1, i, i + 1, &mut swaps);
            j = median(d, j - 1, j, j + 1, &mut swaps);
            k = median(d, k - 1, k, k + 1, &mut swaps);
        }
        j = median(d, i, j, k, &mut swaps);
    }
    (
        j,
        if swaps == 0 {
            1
        } else if swaps == 12 {
            -1
        } else {
            0
        },
    )
}
