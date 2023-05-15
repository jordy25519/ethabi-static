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
pub struct AddressZcp<'a>(pub &'a [u8; 20]);

impl<'a> AsRef<[u8; 20]> for AddressZcp<'a> {
    fn as_ref(&self) -> &'a [u8; 20] {
        self.0
    }
}

impl<'a> AddressZcp<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self(slice_as_array(buf))
    }
}

/// yet another borrowed bytes type...
#[derive(Debug, Default, PartialEq)]
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
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for bool {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        Ok(buf[offset + 31] == 1)
    }
}

impl<'a> DecodeStatic<'a> for U256 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = U256::from(slice_as_array(unsafe {
            buf.get_unchecked(offset..offset + 32_usize)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u128 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = u128::from_be_bytes(*slice_as_array(unsafe {
            buf.get_unchecked(offset + 16..offset + 32_usize)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u64 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = u64::from_be_bytes(*slice_as_array(unsafe {
            buf.get_unchecked(offset + 24..offset + 32_usize)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u32 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = u32::from_be_bytes(*slice_as_array(unsafe {
            buf.get_unchecked(offset + 28..offset + 32_usize)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u16 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = u16::from_be_bytes(*slice_as_array(unsafe {
            buf.get_unchecked(offset + 30..offset + 32_usize)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u8 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        Ok(buf[offset + 31])
    }
}

impl<'a> DecodeStatic<'a> for BytesZcp<'a> {
    fn decode_static(buf: &'a [u8], len_offset: usize) -> Result<Self, ()> {
        let data_offset = len_offset + 32;
        let len = as_usize(&buf[len_offset..]);
        let result = BytesZcp(&buf[data_offset..data_offset + len]);
        Ok(result)
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

        Ok((0..len)
            .map(|i| {
                let next_tail_offset = tail_offset + i * 32;
                // the tail offsets don't include the outer header hence +shift
                as_usize(unsafe { buf.get_unchecked(next_tail_offset..) }) + shift
            })
            .map(|o| T::decode(unsafe { buf.get_unchecked(o..) }).unwrap())
            .collect::<Vec<T>>()
            .into())
    }
}

impl<'a> DecodeStatic<'a> for Vec<BytesZcp<'a>> {
    fn decode_static(buf: &'a [u8], len_offset: usize) -> Result<Self, ()> {
        let len = as_usize(&buf[len_offset..]);
        let tail_offset = len_offset + 32;

        Ok((0..len)
            .map(|i| {
                let next_tail_offset = tail_offset + i * 32;
                // the tail offsets don't include the outer header words hence +64
                as_usize(unsafe { buf.get_unchecked(next_tail_offset..) }) + len_offset + 32
            })
            .map(|o| {
                let res: BytesZcp<'_> = DecodeStatic::decode(&buf[o..]).unwrap();
                res
            })
            .collect())
    }
}

/// Decode a vector of dynamic tuples into the given buffer
pub fn decode_dynamic_list_into<'a, T: DecodeStatic<'a>>(
    buf: &'a [u8],
    offset: usize,
    dst: &mut Vec<T>,
) {
    let len_offset = as_usize(&buf[offset..]);
    let len = as_usize(&buf[len_offset..]);
    let tail_offset = len_offset + 32;
    let shift = 32 + len_offset;

    (0..len)
        .map(|i| {
            let next_tail_offset = tail_offset + i * 32;
            // the tail offsets don't include the outer header hence +shift
            as_usize(unsafe { buf.get_unchecked(next_tail_offset..) }) + shift
        })
        .for_each(|o| {
            dst.push(T::decode(&buf[o..]).unwrap());
        });
}

/// Decode a vector of static `T`s into the given buffer
pub fn decode_static_list_into<'a, T: DecodeStatic<'a>>(
    buf: &'a [u8],
    offset: usize,
    dst: &mut Vec<T>,
) {
    let len_offset = as_usize(&buf[offset..]);
    let len = as_usize(&buf[len_offset..]);
    let tail_offset = len_offset + 32;

    (0..len).for_each(|i| {
        let next_tail_offset = tail_offset + i * 32;
        dst.push(T::decode(&buf[next_tail_offset..]).unwrap());
    })
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
#[derive(Debug, PartialEq)]
pub struct Wrapped<T>(pub T);

impl<'a, T> DecodeStatic<'a> for Wrapped<T>
where
    T: DecodeStatic<'a>,
{
    fn decode_static(buf: &'a [u8], len_offset: usize) -> Result<Self, ()> {
        let data_offset = len_offset + 64; // = bytes offset + bytes length
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
        Ok(result)
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

#[cfg(test)]
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
    #[allow(dead_code)]
    use ethabi::ParamType;
    use ethabi::Token;
    use ethereum_types::U256;
    use hex_literal::hex;

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

        let input = hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");

        let t = Thingy::decode(&input);
        println!("{:?}", t);
        assert!(t.is_ok());
    }

    #[test]
    fn test_ethabi_decode() {
        let input = hex!("00000000000000000000000012345678912345678911111111111111111111110000000000000000000000001234567891234567891111111111111111111222000000000000000000000000000000000000000000000000000000000000303900000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001001122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a10000000000000000000000000000000000000000000000000ff000000000000000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000213370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b33f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a4b05000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001370000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000116000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ff00000000000000000000000000000000000000000000000000000000000000");

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
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 219,
                        86, 223, 166, 126, 253, 44, 225, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 215, 72, 136, 190, 68, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 100, 88, 235, 23
                    ])
                },
                Result3 {
                    success: true,
                    return_data: BytesZcp(&[
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32,
                        97, 86, 209, 214, 142, 148, 78, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 200, 212, 171, 39, 142, 158, 79, 78, 55, 216, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 100, 88, 234, 86
                    ])
                },
                Result3 {
                    success: true,
                    return_data: BytesZcp(&[
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 125,
                        250, 204, 71, 5, 30, 121, 220, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 237, 86, 153, 222, 242, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 1, 44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 44
                    ])
                },
                Result3 {
                    success: true,
                    return_data: BytesZcp(&[
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 105,
                        80, 85, 99, 225, 225, 129, 6, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 2, 143, 23, 78, 238, 76, 51, 223, 120, 207, 83, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 1, 44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 44
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
    fn test() {
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
}
