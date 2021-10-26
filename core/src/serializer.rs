use std::convert::TryInto;

pub type KeyVec = smallvec::SmallVec<[u8; 8]>;

pub trait KeySerializer {
    fn create_key(&self) -> KeyVec;
    fn parse_key(key: &[u8]) -> Self
    where
        Self: std::marker::Sized;
}

pub trait FixedSizeKeySerializer: KeySerializer {
    fn key_size() -> usize;
}

const PANIC_MESSAGE_SIZE: &str = "Key data must fullfill minimal size for type";

impl KeySerializer for Vec<u8> {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(self)
    }

    fn parse_key(key: &[u8]) -> Self {
        Vec::from(key)
    }
}

impl KeySerializer for KeyVec {
    fn create_key(&self) -> KeyVec {
        self.clone()
    }

    fn parse_key(key: &[u8]) -> Self {
        KeyVec::from(key)
    }
}

impl KeySerializer for String {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(self.as_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        let v = String::from_utf8_lossy(key);
        v.to_string()
    }
}

impl KeySerializer for u8 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for u8 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for u16 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for u16 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for u32 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for u32 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for u64 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for u64 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for u128 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for u128 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for usize {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for i8 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for i8 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for i16 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for i16 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for i32 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for i32 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for i64 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for i64 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl KeySerializer for i128 {
    fn create_key(&self) -> KeyVec {
        KeyVec::from_slice(&self.to_be_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl FixedSizeKeySerializer for i128 {
    fn key_size() -> usize {
        std::mem::size_of::<Self>()
    }
}
