//! Native object acceptance and helper classification; raw objects stay intact.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshRole {
    Rejected,
    Geometry,
    Station { index: u8, flags: u8 },
    Weapon { index: u8 },
    Screens,
    UpVector,
}
impl MeshRole {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rejected => "rejected",
            Self::Geometry => "geometry",
            Self::Station { .. } => "station",
            Self::Weapon { .. } => "weapon",
            Self::Screens => "screens",
            Self::UpVector => "up-vector",
        }
    }
}

// Object names are read as unsigned bytes appended as QChars, not UTF-8.
// A one-character QString::toInt failure returns 0; the native caller ignores
// its success flag. Every possible byte therefore yields an accepted 0..9 ID.
fn index(byte: u8) -> u8 {
    if byte.is_ascii_digit() {
        byte - b'0'
    } else {
        0
    }
}

fn station_flags(bytes: &[u8]) -> Option<u8> {
    // QString::toUShort(base 16), restricted to its two Latin-1 characters.
    // Qt trims Unicode space (including NEL/NBSP) and admits a leading '+'.
    let space = |byte: u8| matches!(byte, b'\t'..=b'\r' | b' ' | 0x85 | 0xa0);
    let start = bytes.iter().position(|byte| !space(*byte))?;
    let end = bytes.iter().rposition(|byte| !space(*byte))? + 1;
    let bytes = &bytes[start..end];
    let bytes = bytes.strip_prefix(b"+").unwrap_or(bytes);
    if bytes.is_empty() {
        return None;
    }
    let mut flags = 0u8;
    for byte in bytes {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        flags = flags.checked_mul(16)?.checked_add(digit)?;
    }
    (flags != 0).then_some(flags)
}

impl Mesh {
    /// Classify the patched textured-model loader's acceptance/removal stages.
    /// Retains raw helper data; does not implement station/screen interactions.
    pub fn role(&self, variant: ModelVariant) -> MeshRole {
        self.role_with_screens(variant, &mut [false; 2])
    }

    fn role_with_screens(&self, variant: ModelVariant, screens: &mut [bool; 2]) -> MeshRole {
        let extended = matches!(variant, ModelVariant::N | ModelVariant::NT);
        let special_empty = self.name.starts_with(b"!")
            || self.name.starts_with(b"@")
            || self.name == b"DataOnly"
            || self.name == b"UpVector";
        if self.vertices.is_empty() || (self.faces.is_empty() && (!extended || !special_empty)) {
            return MeshRole::Rejected;
        }
        if self.name.len() > 3
            && self.name[0] == b'@'
            && let Some(flags) = station_flags(&self.name[2..4])
        {
            return MeshRole::Station {
                index: index(self.name[1]),
                flags,
            };
        }
        if self.name.len() > 1 && self.name[0] == b'!' {
            return MeshRole::Weapon {
                index: index(self.name[1]),
            };
        }
        let mut valid_screen_groups = true;
        for group in &self.vertex_groups {
            let side = match group.name.as_deref() {
                Some(b"Screen1") => 0,
                Some(b"Screen2") => 1,
                _ => continue,
            };
            // Native QMap replaces duplicate positions; signed zeros compare
            // equal. Raw memberships and weights remain preserved separately.
            let unique: BTreeSet<_> = group
                .members
                .iter()
                .flatten()
                .map(|member| {
                    member
                        .position
                        .map(|value| if value == 0.0 { 0 } else { value.to_bits() })
                })
                .collect();
            if unique.len() < 4 {
                valid_screen_groups = false;
                break; // Native TestForScreens fails, then tests UpVector.
            }
            screens[side] = true;
        }
        if valid_screen_groups && *screens == [true; 2] {
            return MeshRole::Screens;
        }
        if self.name.len() > 3 && self.name.windows(8).any(|name| name == b"UpVector") {
            return MeshRole::UpVector;
        }
        MeshRole::Geometry
    }
}

impl Model {
    /// Native preprocessing traverses accepted objects in file order and reuses
    /// the two screen descriptors across that traversal. Use this method for
    /// whole models; Mesh::role classifies an isolated object with empty state.
    pub fn mesh_roles(&self, variant: ModelVariant) -> Vec<MeshRole> {
        let mut screens = [false; 2];
        self.meshes
            .iter()
            .map(|mesh| mesh.role_with_screens(variant, &mut screens))
            .collect()
    }
}

#[cfg(test)]
#[path = "roles_tests.rs"]
mod tests;
