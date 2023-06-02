# ethabi-static
[![Build](https://github.com/jordy25519/ethabi-static/actions/workflows/build.yml/badge.svg)](https://github.com/jordy25519/ethabi-static/actions/workflows/build.yml)  
Generates efficient Ethereum ABI decoders at compile time.  
~10-15x speed up vs. `ethabi`  

You probably don't want this crate. It assumes all input is well-formed and sacrifices all else for decoding speed.  
Not feature complete, will accept PRs (could be easily extended to support encoding also).

```rust
use ethabi_static_derive::DecodeStatic;

#[derive(Debug, DecodeStatic)]
struct Foo<'a> {
    a: AddressZcp<'a>,
    b: AddressZcp<'a>,
    c: U256,
    #[ethabi(skip)]
    d: BytesZcp<'a>,
    e: Vec<BytesZcp<'a>>,
    f: FixedBytesZcp<'a, 8>,
}

let foo = Foo::decode(input).unwrap();
```

## Bench
```bash
cargo +nightly bench --features bench --profile=release 
```

```rust
running 2 tests
test bench::test_ethabi_decode               ... bench:     115,997 ns/iter (+/- 27,218)
test bench::test_ethabi_static_decode_bumped ... bench:       4,841 ns/iter (+/- 540)
```
