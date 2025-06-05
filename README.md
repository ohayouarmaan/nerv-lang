# nerv 📀
nerv is a compiled systems programming language inspired by Neon Genesis Evangelion — experimental, sharp, and minimal. Designed to give you low-level power with modern tooling, nerv compiles directly to x86_64 assembly for Linux and macOS.

⚠️ Windows is not currently supported. It may be in the future once inline assembly and syscall support are stable.

### ✨ Features
✅ Compiles to native x86_64 assembly (Linux + macOS)

✅ Links with libc (for now)

✅ Written in Rust

✅ Supports C function calls via extern

✅ Built-in typechecker

🔜 Planned support for inline assembly, syscalls, and a custom standard library

### 🧪 Example
```
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

@returnsPointerToAHeapAllocatedInt(int initialValue) &int {
  dec x &int = malloc(4);
  *x = 45;
  return x;
}
```
### 📦 Building
To build the compiler:
```bash
git clone https://github.com/ohayouarmaan/nerv-lang
cd nerv-lang
Make
```

### 🚀 Roadmap
- [x]  C interop (via extern)
- [x]  Type checking
- [x]  Function compilation
- [ ]  Structs
- [ ]  Inline assembly
- [ ]  Direct syscalls
- [ ]  Windows support
- [ ]  Custom standard library (drop libc)
- [ ]  Modules and imports

### ⚙️ Philosophy

nerv is a systems language that embraces the themes of Neon Genesis Evangelion — introspective, raw, and unafraid to break conventions.

Simplicity wins — No unnecessary abstractions.

You control the machine — Explicit memory, explicit types, explicit control.

Platform-specific precision — Linux and macOS first.

Self-reflective and powerful — Like its namesake, nerv aims to expose the machinery within.
___
📖 License
MIT License