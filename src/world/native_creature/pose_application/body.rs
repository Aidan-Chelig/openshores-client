use super::*;

/// DpBody default constructor: ten zero words at3c..60, false byte6c.
/// Model-data presence is supplied explicitly; no model/bone data is invented.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeDpBodyPoseState {
    pub fields_3c: [u32; 10],
    pub field_6c: bool,
}

impl NativeDpBodyPoseState {
    /// Update state before animation. Animate is required whenever native model
    /// data is nonempty, even for an unchanged pose. Float speed alone does not
    /// contribute to the native changed result and is not stored here.
    pub fn apply(
        &mut self,
        fields: [u32; 10],
        mode: bool,
        speed: f32,
        model_data_nonempty: bool,
        mut animate: impl FnMut(&Self, f32) -> io::Result<()>,
    ) -> io::Result<bool> {
        let changed = self.fields_3c != fields || self.field_6c != mode;
        self.fields_3c = fields;
        self.field_6c = mode;
        if model_data_nonempty {
            animate(self, speed)?;
        }
        Ok(changed)
    }
}

/// AuAnimalia default constructor: zero pose words6c..90, bool40=false, float94=0.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeAnimaliaPoseState {
    pub fields_6c: [u32; 10],
    pub field_40: bool,
    pub field_94: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeAnimaliaPoseChange {
    pub changed: bool,
    /// Raw invalid final word that triggered diagnostics before fallback to8c.
    pub normalized_final_word: Option<u32>,
}

impl NativeAnimaliaPoseState {
    /// Changed is based ONLY on the incoming ten words, BEFORE final-word
    /// normalization. Mode/speed do not affect it. Animate always runs, with
    /// normalized state, even on identical input. Its real timer/model/identity
    /// context must be supplied by the callback, never silently omitted.
    /// State mutations survive callback errors (Rust policy, not native faults).
    pub fn apply(
        &mut self,
        fields: [u32; 10],
        mode: bool,
        speed: f32,
        mut animate: impl FnMut(&Self) -> io::Result<()>,
    ) -> io::Result<NativeAnimaliaPoseChange> {
        let changed = self.fields_6c != fields;
        self.fields_6c = fields;
        self.field_40 = mode;
        self.field_94 = speed;
        let normalized_final_word = if fields[9].wrapping_sub(0x8b) >= 0x15 {
            self.fields_6c[9] = 0x8c;
            Some(fields[9])
        } else {
            None
        };
        animate(self)?;
        Ok(NativeAnimaliaPoseChange {
            changed,
            normalized_final_word,
        })
    }
}
