//! Bounded access to caller-owned, terminated declaration and shader streams.

use core::{ffi::c_void, marker::PhantomData};

use mtld3d_core::{d3d8::declaration::token_words, dxso::operand_token_count};
use mtld3d_shared::InPtrMut;
use mtld3d_types::{D3D_OK, D3DERR_INVALIDCALL, D3DVSD_END};

/// A terminated token stream borrowed for the duration of one COM call.
pub struct Tokens<'a> {
    pointer: *const u32,
    lifetime: PhantomData<&'a [u32]>,
}

impl Tokens<'_> {
    /// Borrows a non-null declaration or shader token stream.
    ///
    /// # Safety
    /// The caller must provide readable tokens through the terminator, including
    /// every payload declared by a token, for the entire borrow.
    pub const unsafe fn new(pointer: *const u32) -> Option<Self> {
        if pointer.is_null() {
            None
        } else {
            Some(Self {
                pointer,
                lifetime: PhantomData,
            })
        }
    }

    const fn word(&self, index: usize) -> Option<u32> {
        if index >= 65536 {
            return None;
        }
        // SAFETY: construction borrows a terminated stream; callers advance by token lengths.
        let pointer = unsafe { self.pointer.add(index) };
        // SAFETY: the stream contract includes this token and allows unaligned caller storage.
        Some(unsafe { pointer.read_unaligned() })
    }

    fn copy(&self, count: usize) -> Option<Vec<u32>> {
        (0..count).map(|index| self.word(index)).collect()
    }

    pub fn declaration(&self) -> Option<Vec<u32>> {
        let mut count = 0;
        while count < 4096 {
            let token = self.word(count)?;
            if token == D3DVSD_END {
                return self.copy(count + 1);
            }
            count = count.checked_add(token_words(token)?)?;
        }
        None
    }

    pub fn shader(&self, vertex: bool) -> Option<Vec<u32>> {
        let version = self.word(0)?;
        if (vertex && version != 0xfffe_0101)
            || (!vertex && !(0xffff_0100..=0xffff_0104).contains(&version))
        {
            return None;
        }
        let mut count = 1;
        loop {
            let token = self.word(count)?;
            count += 1;
            match token & 0xffff {
                0xffff => return self.copy(count),
                0xfffe => count = count.checked_add(((token >> 16) & 0x7fff) as usize)?,
                _ => count += operand_token_count(1, token, |index| self.word(count + index)),
            }
        }
    }
}

/// Copies original tokens using the D3D8 size-query and insufficient-buffer contract.
///
/// # Safety
/// `size` is writable DWORD storage and `output`, when non-null, names at least
/// the input size's number of writable bytes for this call.
pub unsafe fn copy_words(words: &[u32], output: *mut c_void, size: *mut u32) -> i32 {
    // SAFETY: the ABI caller supplies a valid in-out byte count or null.
    let Some(mut size) = (unsafe { InPtrMut::<u32>::opt(size.cast()) }) else {
        return D3DERR_INVALIDCALL;
    };
    let required =
        u32::try_from(words.len() * 4).expect("token streams are bounded to 65536 words");
    if output.is_null() {
        *size = required;
        return D3D_OK;
    }
    if *size < required {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the input byte count proves output space; caller storage cannot alias owned words.
    let output = unsafe { core::slice::from_raw_parts_mut(output.cast::<u8>(), words.len() * 4) };
    for (destination, word) in output.as_chunks_mut::<4>().0.iter_mut().zip(words) {
        destination.copy_from_slice(&word.to_ne_bytes());
    }
    D3D_OK
}
