//! Ethereum ABI static types and impls
use ethereum_types::U256;

/// Provides statically generated Eth ABI decode implementation
pub trait DecodeStatic<'a>: Sized {
    /// Decode an instance from the given abi encoded buf starting at offset
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()>;
    #[cfg(feature = "bump")]
    fn decode_static_into(
        buf: &'a [u8],
        offset: usize,
        bump: &'a bumpalo::Bump,
    ) -> Result<Self, ()> {
        Err(())
    }
    /// Decode an instance from eth abi buffer
    fn decode(buf: &'a [u8]) -> Result<Self, ()> {
        Self::decode_static(buf, 0_usize)
    }
}

#[derive(Debug, PartialEq)]
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
pub struct BytesZcp<'a>(pub &'a [u8]);

impl<'a> AsRef<[u8]> for BytesZcp<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

/// bytes32
pub type Bytes32<'a> = FixedBytesZcp<'a, 32>;
/// bytes16
pub type Bytes16<'a> = FixedBytesZcp<'a, 16>;
/// bytes8
pub type Bytes8<'a> = FixedBytesZcp<'a, 8>;
/// bytes4
pub type Bytes4<'a> = FixedBytesZcp<'a, 4>;

/// bytesN
#[derive(Debug, PartialEq)]
pub struct FixedBytesZcp<'a, const N: usize>(pub &'a [u8; N]);

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

impl<'a> DecodeStatic<'a> for AddressZcp<'a> {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        Ok(AddressZcp::new(buf))
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
            buf.get_unchecked(offset..)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u128 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = u128::from_be_bytes(*slice_as_array(unsafe {
            buf.get_unchecked(offset + 16..)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u64 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = u64::from_be_bytes(*slice_as_array(unsafe {
            buf.get_unchecked(offset + 24..)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u32 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = u32::from_be_bytes(*slice_as_array(unsafe {
            buf.get_unchecked(offset + 28..)
        }));
        Ok(result)
    }
}

impl<'a> DecodeStatic<'a> for u16 {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let result = u16::from_be_bytes(*slice_as_array(unsafe {
            buf.get_unchecked(offset + 30..)
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
        let len = as_usize(unsafe { buf.get_unchecked(len_offset..) });
        let result = BytesZcp(unsafe { buf.get_unchecked(data_offset..data_offset + len) });
        Ok(result)
    }
}

/// An array of dynamic tuples
#[derive(Debug, PartialEq)]
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
        let len: usize = as_usize(&buf[len_offset..]);
        let tail_offset = len_offset + 32;

        let mut items = Vec::with_capacity(len);

        (0..len)
            .map(|i| {
                let next_tail_offset = tail_offset + (i << 5);
                // the tail offsets don't include the outer header hence +shift
                as_usize(unsafe { buf.get_unchecked(next_tail_offset..) }) + tail_offset
            })
            .for_each(|o| items.push(T::decode(unsafe { buf.get_unchecked(o..) }).unwrap()));

        Ok(Self(items))
    }
}

/// helper to decode `T` as a dynamic tuple (default behaviour of `T` as a static tuple)
#[derive(Debug, Default, PartialEq)]
pub struct Tuple<T>(pub T);

// dynamic tuple
impl<'a, T: DecodeStatic<'a>> DecodeStatic<'a> for Tuple<T> {
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        let tail_offset = as_usize(unsafe { buf.get_unchecked(offset..) });
        Ok(Self(T::decode(&buf[tail_offset..]).unwrap()))
    }
}

#[cfg(not(feature = "bump"))]
pub struct Array<T, const D: bool>(pub Vec<T>);
#[cfg(feature = "bump")]
pub struct Array<'a, T, const D: bool>(pub Vec<T, &'a bumpalo::Bump>);

impl<'a, T: DecodeStatic<'a>> DecodeStatic<'a> for Array<'a, T, true> {
    #[cfg(not(feature = "bump"))]
    fn decode_static(buf: &'a [u8], len_offset: usize) -> Result<Self, ()> {
        let len = as_usize(&buf[len_offset..]);
        let tail_offset = len_offset + 32;
        let mut items = Vec::with_capacity(len);

        (0..len)
            .map(|i| {
                let next_tail_offset = tail_offset + i * 32;
                // the tail offsets don't include the length word hence +32
                as_usize(unsafe { buf.get_unchecked(next_tail_offset..) }) + tail_offset
            })
            .for_each(|o| {
                items.push(T::decode(&buf[o..]).unwrap());
            });

        Ok(Self(items))
    }
    #[cfg(feature = "bump")]
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        Err(())
    }
    #[cfg(feature = "bump")]
    fn decode_static_into(
        buf: &'a [u8],
        len_offset: usize,
        bump: &'a bumpalo::Bump,
    ) -> Result<Self, ()> {
        let len = as_usize(unsafe { buf.get_unchecked(len_offset..) });
        let tail_offset = len_offset + 32;
        let mut items = Vec::with_capacity_in(len, bump);

        (0..len)
            .map(|i| {
                let next_tail_offset = tail_offset + (i << 5);
                // the tail offsets don't include the length word hence +32
                as_usize(unsafe { buf.get_unchecked(next_tail_offset..) }) + tail_offset
            })
            .for_each(|o| {
                items.push(T::decode(unsafe{ buf.get_unchecked(o..) }).unwrap());
            });

        Ok(Self(items))
    }
}

impl<'a, T: DecodeStatic<'a>> DecodeStatic<'a> for Array<'a, T, false> {
    #[cfg(not(feature = "bump"))]
    fn decode_static(buf: &'a [u8], len_offset: usize) -> Result<Self, ()> {
        let len = as_usize(&buf[len_offset..]);
        let mut items = Vec::with_capacity(len);
        (0..len).for_each(|i| {
            // the tail offsets don't include the length word hence +32
            let idx = len_offset + 32 + i * 32;
            items.push(DecodeStatic::decode(&buf[idx..]).unwrap());
        });

        Ok(Self(items))
    }
    #[cfg(feature = "bump")]
    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
        Err(())
    }
    #[cfg(feature = "bump")]
    fn decode_static_into(
        buf: &'a [u8],
        len_offset: usize,
        bump: &'a bumpalo::Bump,
    ) -> Result<Self, ()> {
        let len = as_usize(unsafe { buf.get_unchecked(len_offset..) });
        let mut items = Vec::with_capacity_in(len, bump);
        (0..len).for_each(|i| {
            // the tail offsets don't include the length word hence +32
            let idx = len_offset + ((i + 1) << 5);
            items.push(DecodeStatic::decode(&buf[idx..]).unwrap());
        });

        Ok(Self(items))
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
        Ok(Self::new(buf))
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
pub(crate) fn as_usize(buf: &[u8]) -> usize {
    // OPTIMIZATION: nothing sensible should ever be longer than 2 ** 16 so we ignore the other bytes
    // ((unsafe { *buf.get_unchecked(28) } as usize) << 24)
    //     + ((unsafe { *buf.get_unchecked(29) } as usize) << 16)
    ((unsafe { *buf.get_unchecked(30) } as usize) << 8)
        + (unsafe { *buf.get_unchecked(31) } as usize)
}
