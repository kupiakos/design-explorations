use core::arch::asm;

#[inline(always)]
pub fn command(driver: usize, command_number: usize, arg1: usize, arg2: usize) -> usize {
    let result: usize;
    unsafe {
        asm!("svc 2",
            inlateout("r0") driver => result,
            inlateout("r1") command_number => _,
            inlateout("r2") arg1 => _,
            inlateout("r3") arg2 => _,
            options(preserves_flags, nostack),
        );
    }
    result
}

#[inline(always)]
pub fn subscribe<T>(
    driver: usize,
    subscribe_number: usize,
    callback: unsafe extern "C" fn(usize, usize, usize, Option<&T>),
    data: Option<&T>,
) -> usize {
    let result;
    let data = data.map(|x| x as *const T).unwrap_or(core::ptr::null());
    // same: `let data: *const T = core::mem::transmute(data);`
    unsafe {
        asm!(
            "svc 1",
            inlateout("r0") driver => result,
            inlateout("r1") subscribe_number => _,
            inlateout("r2") callback => _,
            inlateout("r3") data => _,
            options(preserves_flags, nostack),
        );
    }
    result
}

#[inline(always)]
#[cfg(target_arch = "arm")]
pub fn yieldk() {
    // Note: A process stops yielding when there is a callback ready to run,
    // which the kernel executes by modifying the stack frame pushed by the
    // hardware. The kernel copies the PC value from the stack frame to the LR
    // field, and sets the PC value to callback to run. When this frame is
    // unstacked during the interrupt return, the effectively clobbers the LR
    // register.
    //
    // At this point, the callback function is now executing, which may itself
    // clobber any of the other caller-saved registers. Thus we mark this inline
    // assembly as conservatively clobbering all caller-saved registers, forcing
    // yield to save any live registers.
    //
    // Upon direct observation of this function, the LR is the only register
    // that is live across the SVC invocation, however, if the yield call is
    // inlined, it is possible that the LR won't be live at all (commonly seen
    // for the `loop { yieldk(); }` idiom) or that other registers are live,
    // thus it is important to let the compiler do the work here.
    //
    // According to the AAPCS: A subroutine must preserve the contents of the
    // registers r4-r8, r10, r11 and SP (and r9 in PCS variants that designate
    // r9 as v6). Thus we must clobber r0-3, r12, and LR
    const YIELD_ID_WAIT: u32 = 1;
    unsafe {
        // copied from libtock-rs/runtime/src/syscalls_impl_arm.rs
        asm!("svc 0",
                 inlateout("r0") YIELD_ID_WAIT => _, // a1
                 // r4-r8 are callee-saved.
                 // r9 is platform-specific. We don't use it in libtock_runtime,
                 // so it is either unused or used as a callee-saved register.
                 // r10 and r11 are callee-saved.

                 // r13 is the stack pointer and must be restored by the callee.
                 // r15 is the program counter.

                 clobber_abi("C"), // a2, a3, a4, ip (r12), lr (r14)
            );
        // asm!("svc 0", out("r0") _, out("r1") _, out("r2") _, out("r3") _, out("r12") _, out("lr") _);
    }
}
