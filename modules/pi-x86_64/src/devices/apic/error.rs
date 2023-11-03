use bitflags::bitflags;

bitflags! {
    pub struct ErrorStatus : u32 {
        const REDIRECTABLE_IPI = 1 << 4;
        const SEND_ILLEGAL_VECTOR = 1 << 5;
        const RECEIVE_ILLEGAL_VECTOR = 1 << 6;
        const ILLEGAL_REGISTER_ADDRESS = 1 << 7;
    }
}