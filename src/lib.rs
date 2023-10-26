mod err;
mod parser;
mod stack;
mod wasm;

mod reflection;
#[cfg(test)]
pub mod testing;

pub(crate) use err::SyntaxError;
pub(crate) use stack::*;
use uuid::Uuid;
#[cfg(feature = "simd")]
use uuid_simd::UuidExt;

pub(crate) fn generate_uuid() -> Uuid {
    #[cfg(test)]
    return testing::uuid::generate_test_uuid();

    #[cfg(not(test))]
    return Uuid::new_v4();
}

pub(crate) fn parse_uuid(text: &str) -> anyhow::Result<Uuid> {
    #[cfg(not(feature = "simd"))]
    let parsed = Uuid::parse_str(text)?;

    #[cfg(feature = "simd")]
    let parsed = Uuid::parse(text.as_bytes())?;

    Ok(parsed)
}
