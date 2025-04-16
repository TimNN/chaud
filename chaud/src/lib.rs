#![doc = include_str!(env!("README"))]

#[derive(Copy, Clone)]
pub struct Handle {}

impl Handle {
    pub fn create0(_: fn() -> u32) -> Self {
        todo!()
    }

    pub fn get(self) -> fn() -> u32 {
        todo!();
    }
}
