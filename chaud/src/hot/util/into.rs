/// Like [`Into`], but posisbly cfg-dependent.
pub trait CfgInto<T> {
    fn cfg_into(self) -> T;
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl CfgInto<usize> for u32 {
    #[expect(clippy::expect_used, reason = "`cfg` ensures this is unreachable")]
    #[inline]
    fn cfg_into(self) -> usize {
        self.try_into().expect("unreachable")
    }
}
