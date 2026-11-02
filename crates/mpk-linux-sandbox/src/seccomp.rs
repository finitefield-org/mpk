//! Shared native privilege drop and seccomp installation for Go probes.

use super::{raw_syscall0, raw_syscall2, raw_syscall3, raw_syscall5};
use std::ptr;

pub(super) fn program(filter: &[libc::sock_filter]) -> libc::sock_fprog {
    libc::sock_fprog {
        len: u16::try_from(filter.len()).expect("bounded syscall filter"),
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
