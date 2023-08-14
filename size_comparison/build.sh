#!/bin/bash

cargo build --release -Z build-std=core --target thumbv7m-none-eabi

function getoldsize() {
    echo $(($(grep '^   [0-9a-f]' disassembly/old/$1.txt | cut -f2 | tr -d '[:space:]' | wc -c) / 2))
}

function getsize() {
    echo $(($(grep '^   [0-9a-f]' disassembly/$1.txt | cut -c11-24 | tr -d '[:space:]' | wc -c) / 2))
}

echo Disassembly Sizes:
for name in futures futures-noinline no_futures; do
    # arm-none-eabi-objdump -d
    llvm-objdump -d target/thumbv7m-none-eabi/release/$name > disassembly/$name.txt
    printf "%-16s %d (old %d)\n" "$name" "$(getsize $name)" "$(getoldsize $name)"
done


