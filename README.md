# nerv

**nerv** is an experimental systems programming language inspired by *Neon Genesis Evangelion*. It's built for people who enjoy understanding what's happening under the hood. Instead of targeting a virtual machine or LLVM, **nerv** compiles directly to x86_64 assembly for Linux and macOS.

It's still in its early stages, but the goal is simple: build a language that stays close to the hardware without sacrificing a pleasant developer experience.

> **Note:** Windows isn't supported yet. It'll likely arrive once inline assembly and direct syscall support are mature enough.

## Features

### Current

* Native x86_64 code generation for Linux and macOS
* Written entirely in Rust
* C interoperability through `extern`
* Built-in type checker
* Currently links against `libc`

### Planned

* Structs
* Inline assembly
* Direct Linux/macOS syscalls
* Modules and imports
* A custom standard library (removing the `libc` dependency)

## Example

```nerv
extern printf(string, int) int;
extern malloc(int) &int;

@main() int {
    dec validPointer &int = returnsPointerToAHeapAllocatedInt();
    dec danglingPointer &int = returnsPointerToAStackVariable();

    printf("Value of valid pointer: %d", *validPointer);
    printf("Value of dangling pointer: %d", *danglingPointer);

    return 0;
}

@returnsPointerToAStackVariable() &int {
    dec a int = 4;
    return &a;
}

@returnsPointerToAHeapAllocatedInt() &int {
    dec x &int = malloc(4);
    *x = 45;
    return x;
}
```

## Building

```bash
git clone https://github.com/ohayouarmaan/nerv-lang
cd nerv-lang
make
```

## Roadmap

* [x] C interoperability (`extern`)
* [x] Static type checking
* [x] Function compilation
* [ ] Structs
* [ ] Inline assembly
* [ ] Direct syscalls
* [ ] Windows support
* [ ] Custom standard library
* [ ] Modules and imports

## Philosophy

I started **nerv** because I wanted to understand what a compiler actually does—from parsing source code all the way to emitting machine instructions. Over time it became much more than a learning project.

The language takes inspiration from the philosophy of *Neon Genesis Evangelion*: expose what's underneath, don't hide complexity, and embrace understanding over convenience.

That translates into a few simple ideas:

* **Keep it small.** Every feature should earn its place.
* **Stay close to the machine.** Memory, pointers, and control flow should be explicit.
* **Be predictable.** What you write should closely match what the CPU executes.
* **Learn by building.** The compiler is designed to be understandable, hackable, and fun to explore.

nerv isn't trying to replace C or Rust. It's an experiment in language design, compiler construction, and systems programming—a playground for anyone curious about how software works beneath the abstractions.

## License

MIT License
