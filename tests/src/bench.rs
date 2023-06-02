extern crate test;
use test::{black_box, Bencher};

use ethabi::ParamType;
use ethereum_types::U256;

use ethabi_static::{
    Bump, AddressZcp, BytesZcp, DecodeStatic, Bytes8,
};

#[bench]
fn test_ethabi_static_decode_bumped(b: &mut Bencher) {
    #[derive(Debug, DecodeStatic)]
    struct Thingy<'a> {
        a: AddressZcp<'a>,
        b: AddressZcp<'a>,
        c: U256,
        d: BytesZcp<'a>,
        e: Vec<BytesZcp<'a>, &'a Bump>,
        f: Bytes8<'a>,
    }
    let input = &hex_literal::hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");
    let mut bump = Bump::with_capacity(4096);

    b.iter(|| {
        for i in 1..100 {
            black_box({
                Thingy::decode_static_into(input, 0_usize, &bump);
            });
        }
    });
}

// #[bench]
// fn test_ethabi_static_decode(b: &mut Bencher) {
//     #[derive(Debug, DecodeStatic)]
//     struct Thingy<'a> {
//         a: AddressZcp<'a>,
//         b: AddressZcp<'a>,
//         c: U256,
//         d: BytesZcp<'a>,
//         e: Vec<BytesZcp<'a>>,
//         f: FixedBytesZcp<'a, 8>,
//     }

//     let input = hex_literal::hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");

//     b.iter(|| {
//         for _ in 1..100 {
//             black_box(Thingy::decode_static(&input, 0_usize));
//         }
//     });
// }

#[bench]
fn test_ethabi_decode(b: &mut Bencher) {
    let input = hex_literal::hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");

    let types = &[
        ParamType::Address,
        ParamType::Address,
        ParamType::Uint(256_usize),
        ParamType::Bytes,
        ParamType::Array(Box::new(ParamType::Bytes)),
        ParamType::FixedBytes(8),
    ];

    b.iter(|| {
        for _ in 1..100 {
            black_box(ethabi::decode(types, &input));
        }
    });
}
