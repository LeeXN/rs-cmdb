use std::io;

/// Apply the strongest sandbox available for the target platform. Linux
/// architectures use a seccomp-BPF deny list. Platforms without an
/// equivalent implementation fail closed instead of silently running the
/// command without isolation.
#[cfg(target_os = "linux")]
pub fn apply_sandbox() -> io::Result<()> {
    install_no_new_privs()?;
    install_seccomp_filter()
}

#[cfg(not(target_os = "linux"))]
pub fn apply_sandbox() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "command sandbox is unavailable on this platform",
    ))
}

#[cfg(target_os = "linux")]
fn install_no_new_privs() -> io::Result<()> {
    let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if ret != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn platform_syscalls() -> io::Result<(u32, &'static [u32])> {
    // Syscall numbers are architecture-specific. The generic table is used
    // by the 64-bit architectures below; x86 has its own numbering.
    #[cfg(target_arch = "x86_64")]
    {
        return Ok((
            0xC000_003E, // AUDIT_ARCH_X86_64
            &[
                56,  // clone
                57,  // fork
                58,  // vfork
                101, // ptrace
                161, // chroot
                165, // mount
                166, // umount2
                169, // reboot
                175, // init_module
                176, // delete_module
                246, // kexec_load
                272, // unshare
                308, // setns
                313, // finit_module
                321, // bpf
                322, // execveat
                435, // clone3
                442, // mount_setattr
            ],
        ));
    }

    #[cfg(target_arch = "x86")]
    {
        return Ok((
            0x4000_0003, // AUDIT_ARCH_I386
            &[
                2,   // fork
                21,  // mount
                26,  // ptrace
                52,  // umount2
                61,  // chroot
                88,  // reboot
                120, // clone
                128, // init_module
                129, // delete_module
                190, // vfork
                217, // pivot_root
                283, // kexec_load
                310, // unshare
                346, // setns
                350, // finit_module
                357, // bpf
                358, // execveat
                435, // clone3
            ],
        ));
    }

    #[cfg(target_arch = "aarch64")]
    {
        return Ok((
            0xC000_00B7, // AUDIT_ARCH_AARCH64
            &[
                39,  // umount2
                40,  // mount
                41,  // pivot_root
                51,  // chroot
                97,  // unshare
                104, // kexec_load
                105, // init_module
                106, // delete_module
                117, // ptrace
                142, // reboot
                220, // clone
                268, // setns
                273, // finit_module
                280, // bpf
                281, // execveat
                435, // clone3
                442, // mount_setattr
            ],
        ));
    }

    #[cfg(target_arch = "riscv64")]
    {
        return Ok((
            0xC000_00F3, // AUDIT_ARCH_RISCV64
            &[
                39, 40, 41, 51, 97, 104, 105, 106, 117, 142, 220, 268, 273, 280, 281, 435, 442,
            ],
        ));
    }

    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "seccomp sandbox is not implemented for this Linux architecture",
    ))
}

#[cfg(target_os = "linux")]
fn install_seccomp_filter() -> io::Result<()> {
    #[repr(C)]
    struct SockFilter {
        code: u16,
        jt: u8,
        jf: u8,
        k: u32,
    }

    #[repr(C)]
    struct SockFprog {
        len: u16,
        filter: *const SockFilter,
    }

    fn stmt(code: u16, k: u32) -> SockFilter {
        SockFilter {
            code,
            jt: 0,
            jf: 0,
            k,
        }
    }

    fn jump(code: u16, k: u32, jt: u8, jf: u8) -> SockFilter {
        SockFilter { code, jt, jf, k }
    }

    const BPF_LD: u16 = 0x00;
    const BPF_JMP: u16 = 0x05;
    const BPF_RET: u16 = 0x06;
    const BPF_W: u16 = 0x00;
    const BPF_ABS: u16 = 0x20;
    const BPF_JEQ: u16 = 0x10;
    const BPF_K: u16 = 0x00;
    const SECCOMP_RET_ALLOW: u32 = 0x7FFF_0000;
    const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;

    let (audit_arch, denied_syscalls) = platform_syscalls()?;
    let mut filter = Vec::with_capacity(4 + denied_syscalls.len() * 2);

    // Reject an unexpected syscall ABI. Without this check, a process using
    // a different ABI could bypass the architecture-specific syscall table.
    filter.push(stmt(BPF_LD | BPF_W | BPF_ABS, 4));
    filter.push(jump(BPF_JMP | BPF_JEQ | BPF_K, audit_arch, 1, 0));
    filter.push(stmt(BPF_RET | BPF_K, SECCOMP_RET_KILL_PROCESS));

    filter.push(stmt(BPF_LD | BPF_W | BPF_ABS, 0));
    for syscall in denied_syscalls {
        // A match executes the following KILL instruction; a non-match skips
        // it and continues with the next syscall comparison.
        filter.push(jump(BPF_JMP | BPF_JEQ | BPF_K, *syscall, 0, 1));
        filter.push(stmt(BPF_RET | BPF_K, SECCOMP_RET_KILL_PROCESS));
    }
    filter.push(stmt(BPF_RET | BPF_K, SECCOMP_RET_ALLOW));

    let prog = SockFprog {
        len: filter.len() as u16,
        filter: filter.as_ptr(),
    };

    const SECCOMP_SET_MODE_FILTER: libc::c_ulong = 1;
    let ret = unsafe {
        libc::syscall(
            libc::SYS_seccomp,
            SECCOMP_SET_MODE_FILTER,
            0 as libc::c_ulong,
            &prog as *const SockFprog,
        )
    };
    if ret != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn supported_linux_architectures_have_a_non_empty_deny_list() {
        let (_, denied) = super::platform_syscalls().unwrap();
        assert!(!denied.is_empty());
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn unsupported_platforms_fail_closed() {
        let error = super::apply_sandbox().expect_err("sandbox must not silently allow commands");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    }
}
