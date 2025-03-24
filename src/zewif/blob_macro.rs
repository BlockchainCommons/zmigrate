#[macro_export]
macro_rules! blob {
    ($name:ident, $size:expr) => {
        pub struct $name($crate::zewif::Blob<$size>);

        impl $name {
            pub fn new(data: [u8; $size]) -> Self {
                Self($crate::zewif::Blob::new(data))
            }

            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            pub fn to_vec(&self) -> Vec<u8> {
                self.0.to_vec()
            }

            pub fn from_slice(data: &[u8]) -> ::anyhow::Result<Self> {
                Ok(Self($crate::zewif::Blob::from_slice(data)?))
            }

            pub fn from_vec(data: Vec<u8>) -> ::anyhow::Result<Self> {
                Ok(Self($crate::zewif::Blob::from_vec(data)?))
            }

            pub fn from_hex(hex: &str) -> Self {
                Self($crate::zewif::Blob::from_hex(hex))
            }

            pub fn reverse(&mut self) {
                self.0.reverse();
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0.eq(&other.0)
            }
        }

        impl Eq for $name {}

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state)
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.0)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self($crate::zewif::Blob::default())
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl From<$name> for Vec<u8> {
            fn from(blob: $name) -> Vec<u8> {
                blob.to_vec()
            }
        }

        impl From<&$name> for Vec<u8> {
            fn from(blob: &$name) -> Vec<u8> {
                blob.to_vec()
            }
        }

        impl From<Vec<u8>> for $name {
            fn from(data: Vec<u8>) -> Self {
                Self::from_vec(data).unwrap()
            }
        }

        impl From<&[u8]> for $name {
            fn from(data: &[u8]) -> Self {
                Self::from_slice(data).unwrap()
            }
        }

        impl $crate::zewif::parser::Parse for $name {
            fn parse(parser: &mut $crate::zewif::parser::Parser) -> ::anyhow::Result<Self>
            where
                Self: Sized,
            {
                let bytes = ::anyhow::Context::with_context(parser.next($size), || {
                    format!("Parsing {}", stringify!($name))
                })?;
                Ok(Self($crate::zewif::Blob::from(bytes)))
            }
        }
    };
}
