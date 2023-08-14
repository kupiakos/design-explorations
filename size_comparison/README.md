# Futures versus no-Futures Size Comparison

Original author: Johnathan Van Why

I spent some time investigating how to implement futures in `libtock-rs`. During
that investigation, I got the impression that futures have quite a bit of
overhead. This comparison exists in order to objectively evaluate whether my
impression is correct -- and if so, how large the overhead is.

I implemented the same application twice: one without using futures, and one
using futures. The implementation is standalone: it does not depend on any
libraries other than the libraries built in to Rust's toolchain (e.g. libcore).
The application blinks an LED until a button is pressed; while the button is
held down the blinking stops.

Build with `./build.sh`

## Results

### Update by @kupiakos

Updated to run on a modern nightly with stable asm.
It somewhat more closely resembles Tock 2 syscalls, though not completely.

TODO: determine why the sizes are different, reproduce the same methodology as @jrvanwhy.

### Summary

Here are the sizes of each relevant section in the app, in bytes:

| Section   | No Futures | Futures |
| --------- | ---------- | ------- |
| `.text`   | 566        | 1018    |
| `.rodata` | 0          | 0       |
| `.data`   | 0          | 80      |
| `.bss`    | 8          | 32      |

`.text` is 80% larger in the futures-based app than in the no-futures app.

### Disassembly Analysis

I've included a disassembly of each app in `disassembly/`. The apps were built
at commit c38bf5dc47d4854eab8f735cb3295172185e7a6f.

#### Unchanged symbols
The following symbols remained the same size between the apps:

* `__aeabi_memclr`: 6 bytes
* `__aeabi_memcpy`: 104 bytes
* `__aeabi_memset`: 84 bytes
* `rust_start`: 40 bytes
* `start`: 60 bytes

This isn't surprising: `start` and `rust_start` are part of the entry point,
which is the same in the apps. The `__aeabi` functions are dependencies of the
entry point and are part of the `compiler-rt` prebuilt library.

#### Removed/shrunk symbols

The following symbols lost size or disappeared between the apps:

* `alarm::set_delay`: 56 to 52 bytes (-4)
* `gpio::interrupt`: 56 to 32 bytes (-24)

I moved a division from `alarm::set_delay` to `alarm::init` while I was
implementing the futures-based version of the app (which came after the
no-futures version).

`app::button_change` was inlined into `gpio::interrupt` in the no-futures
version of the app; the futures-based version used a virtual call that LLVM was
unable to inline. The inlined `app::button_change` is smaller than the `Waker`
lookup and virtual call logic.

#### Growing symbols

The following symbols grew when futures support was added:

* `app::APP`: 2 to 80 bytes (+78)
* `alarm::interrupt`: 44 to 52 bytes (+8)
* `main`: 116 to 172 bytes (+56)

`app::APP` now contains a future executor, a `RawWakerVTable` (for identifying
which sub-future needs to be polled), and a `Waker` (used when one of the
sub-futures completes). `alarm::interrupt` grew when an inlined call to
`app::alarm_fired()` was replaced with an indirect call through a `Waker`.
`main` grew, but everything it calls is inlined so it's unclear whether the
growth is significant.

#### New symbols

The following symbols were added to support futures:

* `app::waker_drop`: 2 bytes (+2)
* `app::waker_wake`: 28 bytes (+2)
* `app::waker_clone`: 12 bytes (+12)
* `gpio::BUTTON_VALUE`: 1 bytes (+1)
* `gpio::WAKER`: 8 bytes (+8)
* `task::waker_drop`: 2 bytes (+2)
* `task::Task::waker_wake`: 4 bytes (+4)
* `task::Task::poll_future`: 360 bytes (+360)
* `task::Task::waker_clone`: 6 bytes (+6)
* `alarm::WAKER`: 8 bytes (+8)
* `alarm::PERIOD`: 4 bytes (+4)
* `alarm::CUR_TIME`: 8 bytes (+8)

These symbols make up the glue that routes Tock's signals to the appropriate
code in the application. This is all virtualization logic, but is almost
certainly larger than the virtualization logic that the Tock kernel uses. Note
that `task::Task::poll_future` inlined `app::AppFuture::poll`, and as such is
not entirely a fixed cost.

### Growth/Scalability Analysis

This example is fairly small, and may not be representative of the impact of
futures on larger Tock apps. In order to predict the impact of futures on larger
Tock apps, we can group the above symbols based on how they are affected by app
complexity.

Unfortunately, one of the largest changes, `task::Task::poll_future`, contains a
mix of code that can be a fixed cost (executor logic), code that grows with the
number of combinators, and code that grows with app complexity. In order to
analyze it, I added `#[inline(never)]` in a few key places; this version of the
app is in `futures-noinline/`. Preventing inlining added 68 bytes to the size of
`.text`, but allows for a better scalability analysis.

#### Fixed Costs

The following should not grow with app complexity:

* `task::waker_drop`: 2 bytes (+2)
* `task::Task::poll_future`: 96 bytes

Total: 98 bytes.

Note that in the current implementation, `task::Task::poll_future` is
monomorphized per task. However, it can be refactored to avoid the
monomorphization.

#### Per-Combinator Costs

The following costs will grow with the number of future combinators used:

* `app::APP`: 2 to 80 bytes (+78)
* `app::waker_drop`: 2 bytes (+2)
* `app::waker_wake`: 28 bytes (+2)
* `app::waker_clone`: 12 bytes (+12)
* `<app::AppFuture as Future>::poll`: 236 bytes (+236)

Total: 330 bytes per combinator.

#### Per-Driver Costs

The following costs will grow with the number of syscall drivers in use:

* `alarm::interrupt`: 44 to 52 bytes (+8)
* `gpio::BUTTON_VALUE`: 1 bytes (+1)
* `gpio::WAKER`: 8 bytes (+8)
* `alarm::WAKER`: 8 bytes (+8)
* `alarm::CUR_TIME`: 8 bytes (+8)

Total: 16 bytes per syscall driver (rough average).
