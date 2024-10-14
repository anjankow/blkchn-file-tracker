use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use std::fmt::{Debug, Display};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub enum EventType {
    AttributeChanged,
    Created,
    Deleted,
    MovedFrom,
    MovedTo,
    Opened,
    Written,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AttributeChanged => write!(f, "AttributeChanged"),
            Self::Created => write!(f, "Created"),
            Self::Deleted => write!(f, "Deleted"),
            Self::MovedFrom => write!(f, "MovedFrom"),
            Self::MovedTo => write!(f, "MovedTo"),
            Self::Opened => write!(f, "Opened"),
            Self::Written => write!(f, "Written"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]

pub struct OffsetDateTime(time::OffsetDateTime);

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct Event {
    pub file_name: String,
    pub received_at: OffsetDateTime,
    pub event_type: EventType,

    pub file_info: Option<FileInfo>,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file_name, self.event_type)
    }
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct FileInfo {
    pub access_ts: Option<OffsetDateTime>,
    pub modify_ts: Option<OffsetDateTime>,
    pub created_ts: Option<OffsetDateTime>,

    pub size: u64,
    pub mode: u32, // libc::mode_t;
}

impl BorshSerialize for OffsetDateTime {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let unix_timestamp = self.0.unix_timestamp_nanos();
        borsh::to_writer(writer, &unix_timestamp)
    }
}

impl BorshDeserialize for OffsetDateTime {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        // find the length of the serialized value
        let schema_container = borsh::schema_container_of::<i128>();
        let serialized_size = schema_container
            .max_serialized_size()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to check serialized size of OffsetDateTime",
                )
            })?;
        let mut buf: Vec<u8> = Vec::with_capacity(serialized_size);
        reader.read_exact(&mut buf)?;

        let unix_timestamp = borsh::from_reader::<R, i128>(reader)?;

        time::OffsetDateTime::from_unix_timestamp_nanos(unix_timestamp)
            .map(|u| OffsetDateTime(u))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

impl BorshSchema for OffsetDateTime {
    fn add_definitions_recursively(
        _definitions: &mut std::collections::BTreeMap<
            borsh::schema::Declaration,
            borsh::schema::Definition,
        >,
    ) {
        // do nothing
    }

    fn declaration() -> borsh::schema::Declaration {
        let schema_container = borsh::schema_container_of::<i64>();
        schema_container
            .declaration()
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_offset_date_time() {
        let time = super::OffsetDateTime(time::OffsetDateTime::now_utc());
        let mut buf: Vec<u8> = Vec::new();
        time.serialize(&mut buf)
            .expect("Serialization failed");
        let res = OffsetDateTime::deserialize(&mut buf.as_slice()).expect("Deserialization failed");
        debug_assert_eq!(res, time);
    }
}
