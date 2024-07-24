# The Rust Forth Compiler

This tool allows me to write Forth "source" that will get transpiled
to an assembler dielect.  It currently supports AT&T-style (so, UNIX,
Linux, etc.) and 64tass for the 6502 and similar family.

# Installation

```
cargo build
```

Then run in place (or with `cargo run`) or `cargo install` it (I
guess) or whatever completes you.

# Invocation

```
rfc --arch ca6502 -d ARCH_65816,ARCH_WDC fth_main.fs > fth.s
```

The above will cause `ARCH_65816` and `ARCH_WDC` to be defined so that
the Forth compile-time word `[DEFINED]` can test for them (or any
arbitrary string).  As an example:

```forth
[DEFINED] ARCH_6502 [IF]

include fth_core_6502.fs

[THEN]

[DEFINED] ARCH_65816 [IF]

include fth_core_65816.fs

[THEN]
```

# Caveats

This isn't a full Forth-2012 or DPANS94.  It's a way to write a Forth
in Forth instead of hand-editing assembler files.

I'm working on rudimentary vocabulary support but that is not yet done.
