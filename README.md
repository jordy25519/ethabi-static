# ethabi-static

Generates efficient Ethereum ABI decoders at compile time  

You probably don't want this crate. It assumes all input is trusted and sacrifices all else for decoding speed.  
Not feature complete, will accept PRs.

```rust
use ethabi_static_derive::DecodeStatic;

#[derive(Debug, DecodeStatic)]
struct Foo<'a> {
    a: AddressZcp<'a>,
    b: AddressZcp<'a>,
    c: U256,
    d: BytesZcp<'a>,
    e: Vec<BytesZcp<'a>>,
    f: FixedBytesZcp<'a, 8>,
}

let foo = Foo::decode(input).unwrap();
```

## Bench
```bash
cd types && cargo +nightly bench --features bench --profile=release 
```

```rust
running 2 tests
test bench::test_ethabi_decode        ... bench:      88,132 ns/iter (+/- 28,037)
test bench::test_ethabi_static_decode ... bench:       5,901 ns/iter (+/- 273)
```