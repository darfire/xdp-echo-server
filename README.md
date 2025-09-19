# xdp-echo-server

## Prerequisites

1. stable rust toolchains: `rustup toolchain install stable`
1. nightly rust toolchains: `rustup toolchain install nightly --component rust-src`
1. (if cross-compiling) rustup target: `rustup target add ${ARCH}-unknown-linux-musl`
1. (if cross-compiling) LLVM: (e.g.) `brew install llvm` (on macOS)
1. (if cross-compiling) C toolchain: (e.g.) [`brew install filosottile/musl-cross/musl-cross`](https://github.com/FiloSottile/homebrew-musl-cross) (on macOS)
1. bpf-linker: `cargo install bpf-linker` (`--no-default-features` on macOS)

## Components

In order to compare userspace and xdp serving, 3 components have been implemented:
* an UDP client
* an XDP server
* a Userspace server

All components have been implemented with rust tools: rust-aya, rust-tokio, etc.

## Build & Run

Use `cargo build`, `cargo check`, etc. as normal. Run your program with:

```shell
cargo run --release --config 'target."cfg(all())".runner="sudo -E"' --bin <executable>
```

For the XDP server:
```shell
cargo run --release --config 'target."cfg(all())".runner="sudo -E"' --bin xdp-echo-server
```

For the Userspace server:
```shell
cargo run --release --config 'target."cfg(all())".runner="sudo -E"' --bin userspace-server
```

For the UDP client, saving the report to a CSV file:
```shell
cargo run --release --config 'target."cfg(all())".runner="sudo -E"' --bin client --output data.csv
```

Cargo build scripts are used to automatically build the eBPF correctly and include it in the
program.

## License

With the exception of eBPF code, xdp-echo-server is distributed under the terms
of either the [MIT license] .

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

### eBPF

All eBPF code is distributed under either the terms of the
[GNU General Public License, Version 2] or the [MIT license], at your
option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the GPL-2 license, shall be
dual licensed as above, without any additional terms or conditions.