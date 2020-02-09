use std::convert::TryInto;

pub trait KeySerializer: Ord {
    fn create_key(&self) -> Vec<u8>;
    fn parse_key(key: &[u8]) -> Self
    where
        Self: std::marker::Sized;
}

const PANIC_MESSAGE_SIZE: &str = "Key data must fullfill minimal size for type";

impl KeySerializer for Vec<u8> {
    fn create_key(&self) -> Vec<u8> {
        self.clone()
    }

    fn parse_key(key: &[u8]) -> Self {
        Vec::from(key)
    }
}

impl KeySerializer for String {
    fn create_key(&self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }

    fn parse_key(key: &[u8]) -> Self {
        let v = String::from_utf8_lossy(key);
        v.to_string()
    }
}

impl KeySerializer for u8 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for u16 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for u32 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for u64 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for u128 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
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
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for i16 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for i32 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for i64 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}

impl KeySerializer for i128 {
    fn create_key(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn parse_key(key: &[u8]) -> Self {
        Self::from_be_bytes(
            key[..std::mem::size_of::<Self>()]
                .try_into()
                .expect(PANIC_MESSAGE_SIZE),
        )
    }
}
