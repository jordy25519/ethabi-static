#![cfg(test)]

use crate::V2_RESULTS;

use ethabi::{ParamType, Token};
use ethabi_static::{AddressZcp, BytesZcp, DecodeStatic, FixedBytesZcp, Tuples, Wrapped};
use ethereum_types::U256;
use hex_literal::hex;

#[test]
fn test_ethabi_static_decode() {
    #[derive(Debug, DecodeStatic)]
    struct Thingy<'a> {
        a: AddressZcp<'a>,
        b: AddressZcp<'a>,
        c: U256,
        d: BytesZcp<'a>,
        e: Vec<BytesZcp<'a>>,
        f: FixedBytesZcp<'a, 8>,
    }

    let input = hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");

    let thingy = Thingy::decode(&input).unwrap();

    let types = &[
        ParamType::Address,
        ParamType::Address,
        ParamType::Uint(256_usize),
        ParamType::Bytes,
        ParamType::Array(Box::new(ParamType::Bytes)),
        ParamType::FixedBytes(8),
    ];
    let thingy_ref = ethabi::decode(types, &input).unwrap();

    assert_eq!(thingy_ref[0], Token::Address(thingy.a.as_ref().into()),);
    assert_eq!(thingy_ref[1], Token::Address(thingy.b.as_ref().into()),);
    assert_eq!(thingy_ref[2], Token::Uint(thingy.c),);
    assert_eq!(thingy_ref[3], Token::Bytes(thingy.d.0.into()),);
    assert_eq!(
        thingy_ref[4],
        Token::Array(vec![
            Token::Bytes(vec![19_u8, 55]),
            Token::Bytes(vec![179_u8, 63]),
            Token::Bytes(vec![164_u8, 176, 80]),
            Token::Bytes(vec![55_u8]),
            Token::Bytes(vec![11_u8]),
            Token::Bytes(vec![22_u8]),
            Token::Bytes(vec![255_u8]),
        ]),
    );
    assert_eq!(
        thingy_ref[5],
        Token::FixedBytes(vec![17, 34, 51, 68, 85, 102, 119, 136]),
    );
}

#[test]
fn decode_vec_of_tuples() {
    #[derive(Debug, DecodeStatic, PartialEq)]
    struct Result3<'a> {
        success: bool,
        return_data: BytesZcp<'a>,
    }

    let out: Tuples<Result3<'_>> = DecodeStatic::decode(V2_RESULTS).unwrap();
    println!("{:?}", out);
    assert_eq!(
        out.0,
        vec![
            Result3 {
                success: true,
                return_data: BytesZcp(&[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 219, 86,
                    223, 166, 126, 253, 44, 225, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 215, 72, 136, 190, 68, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 88, 235,
                    23
                ])
            },
            Result3 {
                success: true,
                return_data: BytesZcp(&[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 97,
                    86, 209, 214, 142, 148, 78, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 200, 212, 171, 39, 142, 158, 79, 78, 55, 216, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100,
                    88, 234, 86
                ])
            },
            Result3 {
                success: true,
                return_data: BytesZcp(&[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 125, 250,
                    204, 71, 5, 30, 121, 220, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 237, 86, 153, 222, 242, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 44, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 1, 44
                ])
            },
            Result3 {
                success: true,
                return_data: BytesZcp(&[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 105, 80,
                    85, 99, 225, 225, 129, 6, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 2, 143, 23, 78, 238, 76, 51, 223, 120, 207, 83, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 1, 44
                ])
            }
        ]
    )
}

#[test]
fn decode_vec_of_tuples_with_unwrapping() {
    #[derive(Debug, DecodeStatic)]
    struct UniswapV2Reserves {
        r0: U256,
        r1: U256,
    }

    #[derive(Debug, DecodeStatic)]
    struct Result3 {
        success: bool,
        return_data: Wrapped<UniswapV2Reserves>,
    }

    let out: Tuples<Result3> = DecodeStatic::decode(V2_RESULTS).expect("it decodes");
    println!("{:?}", out);
}

#[test]
fn decode_vec_of_tuples_with_unwrapping_generic() {
    #[derive(Debug, PartialEq, DecodeStatic)]
    struct UniswapV2Reserves {
        r0: u128,
        r1: u128,
    }

    #[derive(Debug, PartialEq, DecodeStatic)]
    struct GenericResult3<T> {
        #[ethabi(skip)]
        ok: bool,
        data: Wrapped<T>,
    }

    let out: Tuples<GenericResult3<UniswapV2Reserves>> =
        DecodeStatic::decode(V2_RESULTS).expect("it decodes");

    assert_eq!(
        out.0,
        vec![
            GenericResult3 {
                ok: false,
                data: Wrapped(UniswapV2Reserves {
                    r0: 4046096857213803749746,
                    r1: 7521704656452
                })
            },
            GenericResult3 {
                ok: false,
                data: Wrapped(UniswapV2Reserves {
                    r0: 597309834547827068537,
                    r1: 948396339300470631577560
                })
            },
            GenericResult3 {
                ok: false,
                data: Wrapped(UniswapV2Reserves {
                    r0: 2323914906705736621248,
                    r1: 4317895057138
                })
            },
            GenericResult3 {
                ok: false,
                data: Wrapped(UniswapV2Reserves {
                    r0: 1942696770457359681138,
                    r1: 3093580008960478026452819
                })
            }
        ]
    )
}

#[test]
fn uint_decodes() {
    let input = hex!("0000000000000000000000000000000000000000000000000000000000000037000000000000000000000000000000000000000000000000000000000000022b00000000000000000000000000000000000000000000000000000000000015b3000000000000000000000000000000000000000000000000000000000000d9030000000000000000000000000000000000000000000000000000000000087a23");
    #[derive(Debug, PartialEq, DecodeStatic)]
    struct Numero {
        a: u8,
        b: u16,
        c: u32,
        d: u64,
        e: u128,
    }

    let out = Numero::decode(input.as_ref());
    println!("{:?}", out);
    assert_eq!(
        Numero {
            a: 55,
            b: 555,
            c: 5_555,
            d: 55_555,
            e: 555_555,
        },
        out.unwrap(),
    )
}

#[test]
fn statics_list() {
    let input = hex!("00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000022b0000000000000000000000000000000000000000000000000000000000000005000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001");
    #[derive(Debug, PartialEq, DecodeStatic)]
    struct Foo {
        bar: Vec<U256>,
        bools: Vec<bool>,
        val: U256,
    }

    assert_eq!(
        Foo::decode(&input).unwrap(),
        Foo {
            bar: vec![
                U256::from(1_u32),
                U256::from(2_u32),
                U256::from(3_u32),
                U256::from(4_u32),
                U256::from(5_u32),
            ],
            bools: vec![true, false, true],
            val: U256::from(555_u32),
        }
    );
}

#[test]
fn statics_list_2() {
    let input = hex!("000000000000000000000000000000000000000000000000000000000000022b00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001");
    #[derive(Debug, PartialEq, DecodeStatic)]
    struct Foo {
        val: U256,
        bools: Vec<bool>,
    }

    assert_eq!(
        Foo::decode(&input).unwrap(),
        Foo {
            val: U256::from(555_u32),
            bools: vec![true, false, true],
        }
    );
}

#[test]
fn eth_abi_results2() {
    let params = [ParamType::Array(Box::new(ParamType::Tuple(vec![
        ParamType::Bool,
        ParamType::Bytes,
    ])))];
    let out = ethabi::decode(&params, V2_RESULTS);
    println!("{:?}", out);

    if let Token::Array(list) = &out.unwrap()[0] {
        for t in list {
            match t {
                Token::Tuple(ref inner) => {
                    let x = [ParamType::Uint(256), ParamType::Uint(256)];
                    match &inner[1] {
                        Token::Bytes(bytes) => {
                            let out = ethabi::decode(&x, bytes);
                            println!("{:?}", out);
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }
}
