use alloc::vec::Vec;
pub trait ToNullTerminated {
  fn to_null_terminated(&self) -> Vec<u8>;
}

impl ToNullTerminated for &str {
    fn to_null_terminated(&self) -> Vec<u8> {
        let num_bytes_without_nul = self.as_bytes().len();
        let mut v = Vec::with_capacity(num_bytes_without_nul + 1);
        unsafe {
            core::ptr::copy_nonoverlapping(self.as_ptr(), v.as_mut_ptr(), num_bytes_without_nul);
            *v.as_mut_ptr().add(num_bytes_without_nul) = 0;
            v.set_len(num_bytes_without_nul + 1);
        }
        v
    }
}
impl ToNullTerminated for alloc::string::String {
    fn to_null_terminated(&self) -> Vec<u8> {
        (&**self).to_null_terminated()
    }
}
