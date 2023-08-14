#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    unsafe { core::arch::asm!("udf", options(noreturn)) }
}

#[lang = "start"]
fn start<T>(main: fn() -> T, _: isize, _: *const *const u8, _: u8) -> isize {
    main();
    0
}

#[lang = "termination"]
pub trait Termination {}
impl Termination for () {}
