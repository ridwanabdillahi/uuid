//! The implementation for Version 1 UUIDs.
//!
//! Note that you need to enable the `v1` Cargo feature
//! in order to use this module.

use crate::timestamp::Timestamp;
use crate::Uuid;

/// The number of 100 ns ticks between the UUID epoch
/// `1582-10-15 00:00:00` and the Unix epoch `1970-01-01 00:00:00`.
const UUID_TICKS_BETWEEN_EPOCHS: u64 = 0x01B2_1DD2_1381_4000;

/// The Context implementation is specific to Uuids v1 and v6
pub use crate::timestamp::context::Context;

impl Uuid {
    /// Create a new UUID (version 1) using a time value + sequence +
    /// *NodeId*.
    ///
    /// When generating [`Timestamp`]s using a [`ClockSequence`], this function
    /// is only guaranteed to produce unique values if the following conditions
    /// hold:
    ///
    /// 1. The *NodeId* is unique for this process,
    /// 2. The *Context* is shared across all threads which are generating v1
    ///    UUIDs,
    /// 3. The [`ClockSequence`] implementation reliably returns unique
    ///    clock sequences (this crate provides [`Context`] for this
    ///    purpose. However you can create your own [`ClockSequence`]
    ///    implementation, if [`Context`] does not meet your needs).
    ///
    /// The NodeID must be exactly 6 bytes long.
    ///
    /// Note that usage of this method requires the `v1` feature of this crate
    /// to be enabled.
    ///
    /// # Examples
    ///
    /// A UUID can be created from a unix [`Timestamp`] with a
    /// [`ClockSequence`]. RFC4122 requires the clock sequence
    /// is seeded with a random value:
    ///
    /// ```rust
    /// # use uuid::{Timestamp, Context};
    /// # use uuid::Uuid;
    /// # fn random_seed() -> u16 { 42 }
    /// let context = Context::new(random_seed());
    /// let ts = Timestamp::from_unix(&context, 1497624119, 1234);
    ///
    /// let uuid = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "f3b4958c-52a1-11e7-802a-010203040506"
    /// );
    /// ```
    ///
    /// The timestamp can also be created manually as per RFC4122:
    ///
    /// ```
    /// # use uuid::{Uuid, Timestamp, Context, ClockSequence};
    /// let context = Context::new(42);
    /// let ts = Timestamp::from_rfc4122(14976234442241191232, context.generate_sequence(0, 0));
    ///
    /// let uuid = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "b2c1ad40-45e0-1fd6-802a-010203040506"
    /// );
    /// ```
    ///
    /// The timestamp can also just use the current SystemTime
    ///
    /// ```rust
    /// # use uuid::{Timestamp, Context};
    /// # use uuid::Uuid;
    /// let context = Context::new(42);
    /// let ts = Timestamp::now(&context);
    ///
    /// let _uuid = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]);
    /// ```
    ///
    /// [`Timestamp`]: v1/struct.Timestamp.html
    /// [`ClockSequence`]: v1/trait.ClockSequence.html
    /// [`Context`]: v1/struct.Context.html
    pub fn new_v1(ts: Timestamp, node_id: &[u8; 6]) -> Self {
        let (ticks, counter) = ts.to_rfc4122();
        let time_low = (ticks & 0xFFFF_FFFF) as u32;
        let time_mid = ((ticks >> 32) & 0xFFFF) as u16;
        let time_high_and_version = (((ticks >> 48) & 0x0FFF) as u16) | (1 << 12);

        let mut d4 = [0; 8];

        d4[0] = (((counter & 0x3F00) >> 8) as u8) | 0x80;
        d4[1] = (counter & 0xFF) as u8;
        d4[2] = node_id[0];
        d4[3] = node_id[1];
        d4[4] = node_id[2];
        d4[5] = node_id[3];
        d4[6] = node_id[4];
        d4[7] = node_id[5];

        Uuid::from_fields(time_low, time_mid, time_high_and_version, &d4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{std::string::ToString, Variant, Version};
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_new_v1() {
        let time: u64 = 1_496_854_535;
        let time_fraction: u32 = 812_946_000;
        let node = [1, 2, 3, 4, 5, 6];
        let context = Context::new(0);

        let uuid = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);

        assert_eq!(uuid.get_version(), Some(Version::Mac));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
        assert_eq!(
            uuid.hyphenated().to_string(),
            "20616934-4ba2-11e7-8000-010203040506"
        );

        let ts = uuid.get_timestamp().unwrap().to_rfc4122();

        assert_eq!(ts.0 - 0x01B2_1DD2_1381_4000, 14_968_545_358_129_460);

        // Ensure parsing the same UUID produces the same timestamp
        let parsed = Uuid::parse_str("20616934-4ba2-11e7-8000-010203040506").unwrap();

        assert_eq!(
            uuid.get_timestamp().unwrap(),
            parsed.get_timestamp().unwrap()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_new_v1_context() {
        let time: u64 = 1_496_854_535;
        let time_fraction: u32 = 812_946_000;
        let node = [1, 2, 3, 4, 5, 6];

        // This context will wrap
        let context = Context::new((u16::MAX >> 2) - 1);

        let uuid1 = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);

        let time: u64 = 1_496_854_536;

        let uuid2 = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);

        assert_eq!(uuid1.get_timestamp().unwrap().to_rfc4122().1, 16382);
        assert_eq!(uuid2.get_timestamp().unwrap().to_rfc4122().1, 0);

        let time = 1_496_854_535;

        let uuid3 = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);
        let uuid4 = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);

        assert_eq!(uuid3.get_timestamp().unwrap().to_rfc4122().1, 1);
        assert_eq!(uuid4.get_timestamp().unwrap().to_rfc4122().1, 2);
    }
}
