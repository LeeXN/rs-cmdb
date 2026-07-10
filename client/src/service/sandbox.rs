use std::io;

#[cfg(target_os = "linux")]
pub fn apply_sandbox() -> io::Result<()> {
    install_no_new_privs()?;
    install_seccomp_filter()
}

#[cfg(not(target_os = "linux"))]
pub fn apply_sandbox() -> io::Result<()> {
    Ok(())
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
fn install_seccomp_filter() -> io::Result<()> {
    #[repr(C)]
    struct sock_filter {
        code: u16,
        jt: u8,
        jf: u8,
        k: u32,
    }

    #[repr(C)]
    struct sock_fprog {
        len: u16,
        filter: *const sock_filter,
    }

    fn stmt(code: u16, k: u32) -> sock_filter {
        sock_filter { code, jt: 0, jf: 0, k }
    }

    fn jump(code: u16, k: u32, jt: u8, jf: u8) -> sock_filter {
        sock_filter { code, jt, jf, k }
    }

    const BPF_LD: u16 = 0x00;
    const BPF_JMP: u16 = 0x05;
    const BPF_RET: u16 = 0x06;
    const BPF_W: u16 = 0x00;
    const BPF_ABS: u16 = 0x20;
    const BPF_JEQ: u16 = 0x10;
    const BPF_K: u16 = 0x00;

    const AUDIT_ARCH_X86_64: u32 = 0xC000_003E;

    const SECCOMP_RET_ALLOW: u32 = 0x7FFF_0000;
    const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;

    const NR_FORK: u32 = 57;
    const NR_VFORK: u32 = 58;
    const NR_CLONE: u32 = 56;
    const NR_CLONE3: u32 = 435;
    const NR_EXECVEAT: u32 = 320;

    let filter: [sock_filter; 10] = [
        // I0: Load architecture at seccomp_data offset 4
        stmt(BPF_LD | BPF_W | BPF_ABS, 4),
        // I1: If not x86_64, skip to I8 (ALLOW)
        jump(BPF_JMP | BPF_JEQ | BPF_K, AUDIT_ARCH_X86_64, 0, 6),
        // I2: Load syscall number at seccomp_data offset 0
        stmt(BPF_LD | BPF_W | BPF_ABS, 0),
        // I3: If fork, skip to I9 (KILL)
        jump(BPF_JMP | BPF_JEQ | BPF_K, NR_FORK, 5, 0),
        // I4: If vfork, skip to I9 (KILL)
        jump(BPF_JMP | BPF_JEQ | BPF_K, NR_VFORK, 4, 0),
        // I5: If clone, skip to I9 (KILL)
        jump(BPF_JMP | BPF_JEQ | BPF_K, NR_CLONE, 3, 0),
        // I6: If clone3, skip to I9 (KILL)
        jump(BPF_JMP | BPF_JEQ | BPF_K, NR_CLONE3, 2, 0),
        // I7: If execveat, skip to I9 (KILL)
        jump(BPF_JMP | BPF_JEQ | BPF_K, NR_EXECVEAT, 1, 0),
        // I8: Allow all other syscalls
        stmt(BPF_RET | BPF_K, SECCOMP_RET_ALLOW),
        // I9: Kill on denied syscall
        stmt(BPF_RET | BPF_K, SECCOMP_RET_KILL_PROCESS),
    ];

    let prog = sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_ptr(),
    };

    const SECCOMP_SET_MODE_FILTER: libc::c_ulong = 1;

    let ret = unsafe {
        libc::syscall(
            libc::SYS_seccomp,
            SECCOMP_SET_MODE_FILTER,
            0 as libc::c_ulong,
            &prog as *const sock_fprog,
        )
    };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
