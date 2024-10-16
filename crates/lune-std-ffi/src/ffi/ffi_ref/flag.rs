use super::super::bit_mask::*;

pub enum FfiRefFlag {
    Leaked,
    Dereferenceable,
    Readable,
    Writable,
    Offsetable,
    Function,
    Uninit,
}
impl FfiRefFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Leaked => U8_MASK1,
            Self::Dereferenceable => U8_MASK2,
            Self::Writable => U8_MASK3,
            Self::Readable => U8_MASK4,
            Self::Offsetable => U8_MASK5,
            Self::Function => U8_MASK6,
            Self::Uninit => U8_MASK7,
        }
    }
}
