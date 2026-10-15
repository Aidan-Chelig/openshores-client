//! Receive-visible Person constructor subset; not QObject/model allocation.
//! Static evidence and uninitialized-field boundaries: native-person-construction.md.
use super::*;

impl NativePersonSuffixState {
    /// Seed established DaCreature/DaPerson constructor writes for this suffix.
    /// Unlike Default, omitted fields with proven initialization remain known.
    /// Does not construct inherited body/base, signal wiring or runtime effects.
    /// HitNotify and identity1114 remain unobserved until separately established.
    pub fn from_default_constructor_subset() -> Self {
        Self {
            identity: NativePersonIdentity {
                id_10ec: Some(0),
                string_10f0: Some(None),
                id_10f8: Some(0),
                string_1100: Some(None),
                globe_1108: Some(NativeGlobeLlf {
                    field_00: 0.,
                    field_04: 0.,
                }),
                id_1110: Some(0),
                field_1114: None,
                string_1118: Some(None),
            },
            field_660: Some(0),
            state_1130: Some(0),
            // Native elapsed timer starts invalid, not at a fabricated Instant.
            timer_668: None,
            field_838: Some(0),
            field_839: Some(0),
            field_83b: Some(15),
            field_83d: Some(15),
            field_f80: Some(0),
            field_f84: Some(0),
            field_f88: Some(0),
            content_fa0: Some(Vec::new()),
            field_fa8: Some(0),
            field_fa9: Some(0),
            field_fb0: Some(0),
            gear_1390: Some(NativeGear::default()),
            weight_13a8: Some(0.),
            domain_1018: Some(NativeDomain::default()),
            jobs_13b8: Some(NativeJobList::default()),
            inventory_1458: Some(NativeItemHash::default()),
            field_10d0: Some(0),
            field_f58: Some(0),
            field_1408: Some(0.),
            field_f60: Some(0.),
            field_f68: Some(0.),
            field_10e8: Some(0),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests;
