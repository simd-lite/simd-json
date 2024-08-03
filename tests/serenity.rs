#[cfg(all(feature = "serde_impl", feature = "serde"))]
mod test {

    pub(crate) mod snowflake {
        use serde::de::{Error, Visitor};
        use serde::{Deserializer, Serializer};
        use std::fmt;

        pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
            deserializer.deserialize_any(SnowflakeVisitor)
        }

        #[allow(clippy::trivially_copy_pass_by_ref)]
        pub fn serialize<S: Serializer>(id: &u64, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(id)
        }

        struct SnowflakeVisitor;

        impl<'de> Visitor<'de> for SnowflakeVisitor {
            type Value = u64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("string or integer snowflake")
            }

            // Called by formats like TOML.
            fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
                u64::try_from(value).map_err(Error::custom)
            }

            fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(value)
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                value.parse().map_err(Error::custom)
            }
        }
    }

    use serde::{Deserialize, Serialize};
    use serde_json::Value as SerdeValue;
    use simd_json::owned::Value as SimdJsonValue;

    /// An identifier for a Channel
    #[derive(
        Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
    )]
    pub struct ChannelId(#[serde(with = "snowflake")] pub u64);

    #[test]
    fn simd_json_test() {
        let v = SimdJsonValue::from("367538590520967181".to_string());

        let id: ChannelId = simd_json::serde::from_owned_value(v).unwrap();

        println!("{id:?}");
    }

    #[test]
    fn serde_test() {
        let v = SerdeValue::from("367538590520967181".to_string());

        let id: ChannelId = serde_json::from_value(v).unwrap();

        println!("{id:?}");
    }
}
