//! Practical-Go compiler/runtime policy. Native kernel acceptance remains a
//! separate gate; interpreting these BPF instructions is not that acceptance.

const LOAD: u16 = 0x20;
const EQUAL: u16 = 0x15;
const RETURN: u16 = 0x06;
const ALLOW: u32 = 0x7fff_0000;
const KILL: u32 = 0x8000_0000;
const DENY: u32 = 0x0005_0000 | libc::EPERM as u32;
const UNIMPLEMENTED: u32 = 0x0005_0000 | libc::ENOSYS as u32;
const X86_64: u32 = 0xc000_003e;
// Exact original Go1.25 runtime newosproc and syscall fork/exec combinations.
// The PIDFD form is os.StartProcess's default; the other is its supported
// no-pidfd path. No namespace, TLS, sharing or exit-signal bits are optional.
const THREAD: u32 = 0x0005_0f00;
const PROCESS: u32 = 0x0000_4111;
const PROCESS_PIDFD: u32 = 0x0000_5111;

fn instruction(code: u16, jt: u8, jf: u8, k: u32) -> libc::sock_filter {
    libc::sock_filter { code, jt, jf, k }
}

pub(super) fn filter() -> Vec<libc::sock_filter> {
    let mut result = vec![
        instruction(LOAD, 0, 0, 4),
        instruction(EQUAL, 1, 0, X86_64),
        instruction(RETURN, 0, 0, KILL),
        instruction(LOAD, 0, 0, 0),
        instruction(EQUAL, 0, 1, 435), // clone3 has uninspectable pointer flags
        instruction(RETURN, 0, 0, UNIMPLEMENTED),
        instruction(EQUAL, 0, 8, 56), // clone
        instruction(LOAD, 0, 0, 20),
        instruction(EQUAL, 0, 5, 0),
        instruction(LOAD, 0, 0, 16),
        instruction(EQUAL, 2, 0, THREAD),
        instruction(EQUAL, 1, 0, PROCESS),
        instruction(EQUAL, 0, 1, PROCESS_PIDFD),
        instruction(RETURN, 0, 0, ALLOW),
        instruction(RETURN, 0, 0, DENY),
    ];
    // Explicit x86-64 ABI numbers. All sockets, fork/vfork, mount/namespace,
    // ptrace, capability/identity changes, bpf/io_uring and x32/unknown calls
    // take the final denial. Clone remains confined to the exact cases above.
    for number in [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25,
        28, 32, 33, 35, 36, 39, 59, 60, 61, 62, 63, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83,
        84, 85, 86, 87, 89, 90, 91, 95, 96, 97, 98, 99, 100, 102, 104, 107, 108, 110, 111, 115,
        118, 121, 124, 127, 128, 129, 130, 131, 137, 138, 140, 141, 142, 143, 144, 145, 146, 147,
        148, 157, 158, 186, 202, 204, 213, 217, 218, 219, 228, 229, 230, 231, 232, 233, 234, 247,
        257, 262, 263, 264, 265, 267, 269, 270, 271, 273, 274, 280, 281, 285, 286, 287, 289, 290,
        291, 292, 293, 294, 295, 296, 302, 309, 316, 318, 324, 326, 332, 334, 424, 434, 436, 437,
        439,
    ] {
        result.push(instruction(EQUAL, 0, 1, number));
        result.push(instruction(RETURN, 0, 0, ALLOW));
    }
    result.push(instruction(RETURN, 0, 0, DENY));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(arch: u32, syscall: u32, flags: u64) -> u32 {
        let filter = filter();
        let (mut pc, mut accumulator) = (0_usize, 0_u32);
        for _ in 0..filter.len() {
            let row = &filter[pc];
            match row.code {
                LOAD => {
                    accumulator = match row.k {
                        0 => syscall,
                        4 => arch,
                        16 => flags as u32,
                        20 => (flags >> 32) as u32,
                        _ => panic!("unexpected Go BPF load"),
                    }
                }
                EQUAL => pc += usize::from(if accumulator == row.k { row.jt } else { row.jf }),
                RETURN => return row.k,
                _ => panic!("unexpected Go BPF instruction"),
            }
            pc += 1;
        }
        panic!("Go BPF program did not terminate");
    }

    #[test]
    fn go_policy_allows_only_the_exact_pinned_thread_and_process_clone_flags() {
        let allowed = [
            u64::from(THREAD),
            u64::from(PROCESS),
            u64::from(PROCESS_PIDFD),
        ];
        for flags in allowed {
            assert_eq!(evaluate(X86_64, 56, flags), ALLOW);
            for bit in 0..64 {
                let changed = flags ^ (1 << bit);
                assert_eq!(
                    evaluate(X86_64, 56, changed),
                    if allowed.contains(&changed) {
                        ALLOW
                    } else {
                        DENY
                    },
                    "flags {changed:x}"
                );
            }
        }
        for flags in [0, 17, 0x003d_0f00, 0xffff_ffff_ffff_ffff] {
            assert_eq!(evaluate(X86_64, 56, flags), DENY);
        }
        assert_eq!(evaluate(X86_64, 435, 0), UNIMPLEMENTED);
    }

    #[test]
    fn go_policy_preserves_original_initial_pidfd_probe() {
        // Complete original Go1.25 syscall.doCheckClonePidfd uses
        // CLONE_VFORK | CLONE_VM | CLONE_PIDFD without an exit signal.
        assert_eq!(
            evaluate(X86_64, 56, 0x5100),
            ALLOW,
            "the original initial pidfd probe must not force a no-pidfd fallback"
        );
    }

    #[test]
    fn go_policy_closes_network_namespace_privilege_and_unknown_abi_calls() {
        assert_eq!(evaluate(0xc000_00b7, 0, 0), KILL);
        for syscall in [
            41,
            42,
            43,
            44,
            45,
            49,
            50,
            53,
            57,
            58,
            101,
            105,
            106,
            165,
            166,
            175,
            246,
            272,
            288,
            298,
            299,
            307,
            308,
            310,
            311,
            317,
            321,
            322,
            425,
            426,
            427,
            0x4000_0000,
            u32::MAX,
        ] {
            assert_eq!(evaluate(X86_64, syscall, 0), DENY, "syscall {syscall}");
        }
        for syscall in [
            0, 1, 9, 13, 14, 59, 60, 61, 202, 228, 231, 233, 247, 257, 281, 291, 293, 302, 316,
            318, 332, 424, 434, 436,
        ] {
            assert_eq!(evaluate(X86_64, syscall, 0), ALLOW, "syscall {syscall}");
        }
    }
}
