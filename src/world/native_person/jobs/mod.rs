use super::*;
mod receive;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeJobBody {
    Assassin {
        string_10: Option<String>,
        id_18: u32,
        string_20: Option<String>,
        field_28: i32,
    },
    InterdictSystem {
        string_10: Option<String>,
        id_1c: u32,
        string_20: Option<String>,
        field_18: i32,
    },
    Story {
        string_10: Option<String>,
        string_18: Option<String>,
        ignored_wire_byte: u8,
        field_20: i32,
        string_28: Option<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJob {
    pub kind: u8,
    pub time_08: i64,
    pub body: NativeJobBody,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeJobList(pub Vec<NativeJob>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeJobStop {
    pub unknown_type: u8,
    pub unreceived_count: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "inspect native null-factory stop; remaining jobs were not skipped"]
pub struct NativeJobListReceive {
    pub declared_count: u16,
    pub stop: Option<NativeJobStop>,
    pub consumed_bytes: usize,
}

#[cfg(test)]
mod tests;
