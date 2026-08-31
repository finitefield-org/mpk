//! Finite Java candidate syscall policy. Kernel enforcement is exercised by
//! the explicit native Linux gate, never inferred from an emulated JDK run.

use super::{raw_syscall0, raw_syscall2, raw_syscall3, raw_syscall5};
use std::ptr;

const LOAD: u16 = 0x20; // BPF_LD | BPF_W | BPF_ABS
const EQUAL: u16 = 0x15; // BPF_JMP | BPF_JEQ | BPF_K
const RETURN: u16 = 0x06; // BPF_RET | BPF_K
const ALLOW: u32 = 0x7fff_0000;
const KILL: u32 = 0x8000_0000;
const DENY: u32 = 0x0005_0000 | libc::EPERM as u32;
const UNIMPLEMENTED: u32 = 0x0005_0000 | libc::ENOSYS as u32;
const X86_64: u32 = 0xc000_003e;
// glibc pthread_create's shared-process flags. Process creation, vfork,
// namespace creation, exit signals, and unknown/high bits are never allowed.
const PTHREAD_FLAGS: u32 = 0x003d_0f00;

fn instruction(code: u16, jt: u8, jf: u8, k: u32) -> libc::sock_filter {
    libc::sock_filter { code, jt, jf, k }
}

pub(super) fn filter() -> Vec<libc::sock_filter> {
    let mut result = vec![
        instruction(LOAD, 0, 0, 4), // seccomp_data.arch
        instruction(EQUAL, 1, 0, X86_64),
        instruction(RETURN, 0, 0, KILL),
        instruction(LOAD, 0, 0, 0),    // seccomp_data.nr
        instruction(EQUAL, 0, 1, 435), // clone3: cannot inspect pointer flags
        instruction(RETURN, 0, 0, UNIMPLEMENTED),
        instruction(EQUAL, 0, 6, 56), // clone
        instruction(LOAD, 0, 0, 20),  // args[0] high word
        instruction(EQUAL, 0, 3, 0),
        instruction(LOAD, 0, 0, 16), // args[0] low word
        instruction(EQUAL, 0, 1, PTHREAD_FLAGS),
        instruction(RETURN, 0, 0, ALLOW),
        instruction(RETURN, 0, 0, DENY),
    ];
    // x86-64 syscall numbers, deliberately not host-architecture libc values.
    // Unknown calls (including x32), all sockets, fork/vfork, ptrace, mounts,
    // namespaces, kernel modules, bpf and io_uring take the final denial.
    for number in [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25,
        28, 32, 33, 35, 36, 39, 59, 60, 61, 62, 63, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83,
        84, 85, 87, 89, 90, 91, 95, 96, 97, 98, 99, 100, 102, 104, 107, 108, 110, 111, 115, 118,
        121, 124, 127, 128, 129, 130, 131, 137, 138, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        157, 158, 186, 202, 204, 217, 218, 219, 228, 229, 230, 231, 234, 257, 262, 263, 264, 265,
        267, 269, 270, 271, 273, 274, 280, 281, 285, 286, 287, 289, 290, 291, 292, 293, 294, 295,
        296, 302, 309, 318, 324, 332, 334, 436, 437, 439,
    ] {
        result.push(instruction(EQUAL, 0, 1, number));
        result.push(instruction(RETURN, 0, 0, ALLOW));
    }
    result.push(instruction(RETURN, 0, 0, DENY));
    result
}

pub(super) fn program(filter: &[libc::sock_filter]) -> libc::sock_fprog {
    libc::sock_fprog {
        len: u16::try_from(filter.len()).expect("bounded Java filter"),
        filter: filter.as_ptr().cast_mut(),
    }
}

#[repr(C)]
struct CapHeader {
    version: u32,
    pid: i32,
}
#[repr(C)]
struct CapData {
    effective: u32,
    permitted: u32,
    inheritable: u32,
}

// Called only on the syscall-only child path; no allocation, locks, libc
// wrappers or unwinding are permitted here. All filter storage predates clone.
pub(super) fn install(program: *const libc::sock_fprog) -> bool {
    if !cfg!(target_arch = "x86_64")
        || raw_syscall0(libc::SYS_getuid) != 65534
        || raw_syscall0(libc::SYS_geteuid) != 65534
        || raw_syscall0(libc::SYS_getgid) != 65534
        || raw_syscall0(libc::SYS_getegid) != 65534
        || raw_syscall2(libc::SYS_getgroups, 0, 0) != 0
    {
        return false;
    }
    // Empty the bounding set while CAP_SETPCAP is still present. Stop only at
    // the first unsupported capability and require an actual finite bound.
    let mut last_found = false;
    for capability in 0..64 {
        let result = raw_syscall5(libc::SYS_prctl, 24, capability, 0, 0, 0); // PR_CAPBSET_DROP
        if result == -i64::from(libc::EINVAL) {
            last_found = true;
            break;
        }
        if result != 0 {
            return false;
        }
    }
    if !last_found
        || raw_syscall5(libc::SYS_prctl, 47, 4, 0, 0, 0) != 0 // PR_CAP_AMBIENT_CLEAR_ALL
        || raw_syscall5(libc::SYS_prctl, 38, 1, 0, 0, 0) != 0
    // PR_SET_NO_NEW_PRIVS
    {
        return false;
    }
    let header = CapHeader {
        version: 0x2008_0522,
        pid: 0,
    };
    let data = [
        CapData {
            effective: 0,
            permitted: 0,
            inheritable: 0,
        },
        CapData {
            effective: 0,
            permitted: 0,
            inheritable: 0,
        },
    ];
    if raw_syscall2(
        libc::SYS_capset,
        ptr::from_ref(&header) as usize,
        data.as_ptr() as usize,
    ) != 0
    {
        return false;
    }
    if raw_syscall3(libc::SYS_seccomp, 1, 0, program as usize) != 0 {
        return false;
    }
    // Challenge the installed kernel policy before exec, without sending a
    // packet or creating a process. Flags/pointers here cannot create a child.
    raw_syscall3(
        libc::SYS_socket,
        libc::AF_INET as usize,
        libc::SOCK_STREAM as usize,
        0,
    ) == -i64::from(libc::EPERM)
        && raw_syscall3(
            libc::SYS_socket,
            libc::AF_UNIX as usize,
            libc::SOCK_STREAM as usize,
            0,
        ) == -i64::from(libc::EPERM)
        && raw_syscall2(libc::SYS_clone3, 0, 0) == -i64::from(libc::ENOSYS)
        && super::raw_syscall1(libc::SYS_unshare, 0) == -i64::from(libc::EPERM)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(arch: u32, syscall: u32, flags: u64) -> u32 {
        let filter = filter();
        let (mut pc, mut accumulator) = (0_usize, 0_u32);
        loop {
            let row = &filter[pc];
            match row.code {
                LOAD => {
                    accumulator = match row.k {
                        0 => syscall,
                        4 => arch,
                        16 => flags as u32,
                        20 => (flags >> 32) as u32,
                        _ => panic!("unknown BPF load"),
                    }
                }
                EQUAL => pc += usize::from(if accumulator == row.k { row.jt } else { row.jf }),
                RETURN => return row.k,
                _ => panic!("unknown BPF instruction"),
            }
            pc += 1;
        }
    }

    #[test]
    fn java_policy_closes_architecture_network_privilege_and_clone_escapes() {
        assert_eq!(evaluate(0xc000_00b7, 0, 0), KILL);
        assert_eq!(evaluate(X86_64, 0x4000_0000, 0), DENY);
        assert_eq!(evaluate(X86_64, 435, 0), UNIMPLEMENTED);
        assert_eq!(evaluate(X86_64, 56, u64::from(PTHREAD_FLAGS)), ALLOW);
        for bit in 0..64 {
            assert_eq!(
                evaluate(X86_64, 56, u64::from(PTHREAD_FLAGS) ^ (1 << bit)),
                DENY
            );
        }
        for syscall in [
            41, 42, 43, 44, 45, 49, 50, 53, 57, 58, 101, 105, 106, 165, 166, 175, 246, 272, 288,
            298, 308, 321, 322, 425, 426, 427,
        ] {
            assert_eq!(evaluate(X86_64, syscall, 0), DENY, "syscall {syscall}");
        }
        for syscall in [
            0, 1, 9, 10, 13, 14, 59, 60, 202, 218, 228, 231, 257, 273, 302, 318, 332, 334,
        ] {
            assert_eq!(evaluate(X86_64, syscall, 0), ALLOW, "syscall {syscall}");
        }
    }
}
