//! Ethereum ABI static types and impls
#![cfg_attr(feature = "bench", feature(test))]

use ethereum_types::U256;

/// Provides statically generated Eth ABI decode implementation
pub trait DecodeStatic<'a>: Sized {
    /// Decode an instance from the given abi encoded buf starting at offset
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()>;
    /// Decode an instance from eth abi buffer
    fn decode(buf: &'a [u8]) -> Result<Self, ()> {
        Self::decode_static(buf, 0_usize)
    }
}

#[derive(Debug)]
pub struct AddressZcp<'a>(&'a [u8; 20]);

impl<'a> AddressZcp<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self(slice_as_array(buf))
    }
}

/// yet another borrowed bytes type...
#[derive(Debug, Default, Clone, Copy)]
pub struct BytesZcp<'a>(&'a [u8]);

impl<'a> AsRef<[u8]> for BytesZcp<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

/// bytesN
#[derive(Debug)]
pub struct FixedBytesZcp<'a, const N: usize>(&'a [u8; N]);

impl<'a, const N: usize> FixedBytesZcp<'a, N> {
    fn new(val: &'a [u8]) -> Self {
        Self(slice_as_array(val))
    }
}

/// T[N]
#[derive(Debug)]
pub struct FixedArrayZcp<'a, const N: usize, T>(&'a [T; N]);

/// Cast &[T] to &[T; N] w/out runtime checks
fn slice_as_array<T, const N: usize>(slice: &[T]) -> &[T; N] {
    unsafe { &*(slice as *const [T] as *const [T; N]) }
}

fn as_usize(buf: &[u8]) -> usize {
    // OPTIMIZATION: nothing sensible should ever be longer than 2 ** 16 so we ignore the other bytes
    // ((unsafe { *buf.get_unchecked(28) } as usize) << 24)
    //     + ((unsafe { *buf.get_unchecked(29) } as usize) << 16)
    ((unsafe { *buf.get_unchecked(30) } as usize) << 8)
        + (unsafe { *buf.get_unchecked(31) } as usize)
}

impl<'a> DecodeStatic<'a> for AddressZcp<'a> {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = AddressZcp::new(&buf[offset + 12..offset + 32]);
        return Ok(result);
    }
}

impl<'a> DecodeStatic<'a> for bool {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        Ok(buf[offset + 31] == 1)
    }
}

impl<'a> DecodeStatic<'a> for U256 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = U256::from(slice_as_array(&unsafe {
            buf.get_unchecked(offset..offset + 32_usize)
        }));
        return Ok(result);
    }
}

impl<'a> DecodeStatic<'a> for BytesZcp<'a> {
    fn decode_static(buf: &'a [u8], len_offset: usize) -> Result<Self, ()> {
        let data_offset = len_offset + 32;
        let len = as_usize(&buf[len_offset..]);
        let result = BytesZcp(&buf[data_offset..data_offset + len]);
        return Ok(result);
    }
}

/// An array of dynamic tuples
#[derive(Debug)]
pub struct Tuples<T>(pub Vec<T>);

impl<T> From<Vec<T>> for Tuples<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<'a, T> DecodeStatic<'a> for Tuples<T>
where
    T: DecodeStatic<'a>,
{
    /// Assumes array of tuples
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let len_offset = as_usize(&buf[offset..]);
        let len = as_usize(&buf[len_offset..]);
        let tail_offset = len_offset + 32;
        let shift = 32 + len_offset;

        return Ok((0..len)
            .map(|i| {
                let next_tail_offset = tail_offset + i * 32;
                // the tail offsets don't include the outer header hence +shift
                as_usize(&unsafe { buf.get_unchecked(next_tail_offset..) }) + shift
            })
            .map(|o| T::decode(&unsafe { buf.get_unchecked(o..) }).unwrap())
            .collect::<Vec<T>>()
            .into());
    }
}

impl<'a> DecodeStatic<'a> for Vec<BytesZcp<'a>> {
    fn decode_static(buf: &'a [u8], len_offset: usize) -> Result<Self, ()> {
        let len = as_usize(&buf[len_offset..]);
        let tail_offset = len_offset + 32;

        return Ok((0..len)
            .map(|i| {
                let next_tail_offset = tail_offset + i * 32;
                // the tail offsets don't include the outer header words hence +64
                as_usize(&unsafe { buf.get_unchecked(next_tail_offset..) }) + len_offset + 32
            })
            .map(|o| {
                let res: BytesZcp<'_> = DecodeStatic::decode(&buf[o..]).unwrap();
                res
            })
            .collect());
    }
}

/// Helper type meaning a type encoded as `bytes` should be decoded as a `T`
///  E.g. the makerdao multicall contract returns ABI encoded results from proxy calls
///
/// ``ignore
///     struct ContractResult<'a> {
///         a: BytesZcp<'a>,
///         b: AddressZcp<'a>,
///     }
///     struct Result<'a> {
///         success: bool,
///         return_data: Wrapped<ContractResult<'a>>,
///     }
/// ```
#[derive(Debug)]
pub struct Wrapped<T>(pub T);

impl<'a, T> DecodeStatic<'a> for Wrapped<T>
where
    T: DecodeStatic<'a>,
{
    fn decode_static(buf: &'a [u8], len_offset: usize) -> Result<Self, ()> {
        let data_offset = len_offset + 32;
        let len = as_usize(&buf[len_offset..]);
        Ok(Wrapped(T::decode(&buf[data_offset..data_offset + len])?))
    }
}

impl<'a, T, const N: usize> DecodeStatic<'a> for [T; N]
where
    T: Default + DecodeStatic<'a>,
    [T; N]: Default,
{
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        //let tail_offset = N * 32;
        let tail = &buf[offset..];
        let mut tokens: [T; N] = Default::default();
        let mut new_offset = 0;
        for idx in 0..N {
            let res = T::decode_static(tail, new_offset)?;
            new_offset += 32; // static only
            tokens[idx] = res;
        }
        Ok(tokens)
    }
}

impl<'a, const N: usize> DecodeStatic<'a> for FixedBytesZcp<'a, N> {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = Self::new(&buf[offset..offset + 32]);
        return Ok(result);
    }
}

// impl<'a, A> DecodeStatic<'a> for SmallVec<A>
// where
//     A: Array,
//     <A as Array>::Item: DecodeStatic<'a>,
// {
//     fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
//         let len_offset = as_usize(&buf[offset..offset + 32]);
//         let len = as_usize(&buf[len_offset..len_offset + 32]);
//         let tail_offset = len_offset + 32;
//         let tail = &buf[tail_offset..];
//         let mut tokens = SmallVec::with_capacity(len);
//         let mut new_offset = 0;
//         for _ in 0..len {
//             let res = <A as Array>::Item::decode_static(tail, new_offset)?;
//             new_offset += 32;
//             tokens.push(res);
//         }
//         Ok(tokens)
//     }
// }

const V2_RESULTS: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 4, // heads
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 2, 224, // tails
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, // success
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    64, // offset
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    96, // l
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 219, 86, 223, 166, 126,
    253, 44, 225, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 6, 215, 72, 136, 190, 68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 100, 88, 235, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // success
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    64, // offset
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    96, // l
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 97, 86, 209, 214, 142,
    148, 78, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 200, 212, 171,
    39, 142, 158, 79, 78, 55, 216, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 100, 88, 234, 86, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // success
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    64, // offset
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    128, // l
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 125, 250, 204, 71, 5, 30,
    121, 220, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
    237, 86, 153, 222, 242, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 1, 44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // success
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    64, //offset
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    128, // l
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 105, 80, 85, 99, 225, 225,
    129, 6, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 143, 23, 78,
    238, 76, 51, 223, 120, 207, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 44,
];

#[cfg(feature = "bench")]
mod bench {
    #[deny(soft_unstable)]
    extern crate test;
    use test::{black_box, Bencher};

    use ethabi::{Bytes, ParamType};
    use ethereum_types::{Address, H64, U256};

    use crate::{AddressZcp, BytesZcp, DecodeStatic, FixedBytesZcp, Tuples, V2_RESULTS};
    use ethabi_static_derive::DecodeStatic;

    #[bench]
    fn test_ethabi_static_decode(b: &mut Bencher) {
        #[derive(Debug, DecodeStatic)]
        struct Thingy<'a> {
            a: AddressZcp<'a>,
            b: AddressZcp<'a>,
            c: U256,
            d: BytesZcp<'a>,
            e: Vec<BytesZcp<'a>>,
            f: FixedBytesZcp<'a, 8>,
        }

        // 1.1µs - 3.5µs
        let input = hex_literal::hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");

        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..100 {
                black_box(Thingy::decode_static(&input, 0_usize));
            }
        });
    }

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
            // Inner closure, the actual test
            for _ in 1..100 {
                black_box(ethabi::decode(types, &input));
            }
        });
    }
}

#[cfg(test)]
mod test {
    use ethabi::ParamType;
    use ethereum_types::U256;

    use crate::{AddressZcp, BytesZcp, DecodeStatic, FixedBytesZcp, Tuples, Wrapped, V2_RESULTS};
    use ethabi_static_derive::DecodeStatic;

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

        let input = hex_literal::hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");

        // TODO: Tuples<x> working but this isnt...
        let t = Thingy::decode(&input);
        println!("{:?}", t);
        assert!(t.is_ok());
    }

    #[test]
    fn test_ethabi_decode() {
        let input = hex_literal::hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");

        let types = &[
            ParamType::Address,
            ParamType::Address,
            ParamType::Uint(256_usize),
            ParamType::Bytes,
            ParamType::Array(Box::new(ParamType::Bytes)),
            ParamType::FixedBytes(8),
        ];

        let t = ethabi::decode(types, &input);
        println!("{:?}", t);
        assert!(t.is_ok());
    }

    #[test]
    fn decode_vec_of_tuples() {
        #[derive(Debug, DecodeStatic)]
        struct Result3<'a> {
            success: bool,
            return_data: BytesZcp<'a>,
        }

        let out: Tuples<Result3<'_>> = DecodeStatic::decode(V2_RESULTS).unwrap();
        println!("{:?}", out);
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
        #[derive(Debug, DecodeStatic)]
        struct UniswapV2Reserves {
            r0: U256,
            r1: U256,
        }

        #[derive(Debug, DecodeStatic)]
        struct GenericResult3<T> {
            ok: bool,
            data: Wrapped<T>,
        }

        let out: Tuples<GenericResult3<UniswapV2Reserves>> =
            DecodeStatic::decode(V2_RESULTS).expect("it decodes");
        println!("{:?}", out);
    }
}
