use super::*;

impl NativePersonSuffixState {
    /// Caller supplies real inherited construction/retained values for the two
    /// read-modify-write fields. Other fields remain explicit observations.
    pub fn with_prior_fields(field_660: u16, state_1130: u32) -> Self {
        Self {
            field_660: Some(field_660),
            state_1130: Some(state_1130),
            ..Default::default()
        }
    }

    /// No pre-scan, opaque skipping, or successful no-op hooks. Unsupported
    /// nested readers fail at their native stage; earlier mutations remain.
    /// Locks/Qt failed-stream parity are not claimed by this exclusive Rust API.
    pub fn receive_suffix(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
        mut now: impl FnMut() -> Instant,
    ) -> io::Result<NativePersonReceive> {
        let length = input.remaining();
        let flags_a = input.read_u8()?;
        let flags_b = if flags_a & 0x80 != 0 {
            input.read_u8()?
        } else {
            0
        };
        if flags_a & 1 != 0 {
            self.identity.receive(input, profile)?;
        }
        if flags_a & 2 != 0 {
            self.hit_notify.receive(input, profile)?;
        }
        let mut timer_restarted = false;
        if flags_a & 4 != 0 {
            let packed = input.read_u16()?;
            let old = self
                .field_660
                .ok_or_else(|| unsupported("Person prior field660 is unknown"))?;
            if old != packed & 0x7fff {
                self.field_660 = Some(packed & 0x7fff);
                self.timer_668 = Some(now());
                timer_restarted = true;
            }
            self.field_839 = Some((packed >> 15) as u8);
        }
        let flags_c = if flags_a & 8 != 0 {
            input.read_u8()?
        } else {
            0
        };
        self.field_838 = Some(flags_c & 0x3f);
        if flags_a & 0x10 != 0 {
            let packed = input.read_u8()?;
            self.field_f80 = Some(input.read_i32()?);
            self.field_f84 = Some(packed >> 7);
            self.field_f88 = Some(u32::from((packed >> 3) & 3));
        }
        if flags_a & 0x20 != 0 {
            self.field_fa8 = Some(input.read_u8()?);
            let packed = input.read_u8()?;
            self.field_fa9 = Some(packed & 0x0f);
            let contents = self.content_fa0.get_or_insert_with(Vec::new);
            contents.clear();
            if packed & 0x80 != 0 {
                let count = input.read_u8()?;
                for _ in 0..count {
                    contents.push(input.read_u8()?);
                }
            }
            self.field_fb0 = Some(u32::from(input.read_u8()?));
        }
        let gear = if flags_a & 0x40 != 0 {
            let gear = self.gear_1390.get_or_insert_with(Default::default);
            let report = gear.receive(input)?;
            if !gear.0.is_empty() {
                return Err(unsupported(
                    "native nonempty Gear::Weight item virtual+298 is not implemented",
                ));
            }
            self.weight_13a8 = Some(0.);
            Some(report)
        } else {
            None
        };
        if flags_b & 1 != 0 {
            self.domain_1018
                .get_or_insert_with(Default::default)
                .receive(input)?;
        }
        let jobs = if flags_b & 4 != 0 {
            Some(
                self.jobs_13b8
                    .get_or_insert_with(Default::default)
                    .receive_qt7_with_profile(input, profile)?,
            )
        } else {
            None
        };
        if flags_b & 8 != 0 {
            self.field_10d0 = Some(input.read_i32()?);
            self.field_f58 = Some(input.read_i16()?);
        }
        if flags_b & 0x10 != 0 {
            self.field_1408 = Some(double(input, profile)?);
        }
        if flags_b & 0x20 != 0 {
            self.field_f60 = Some(double(input, profile)?);
            self.field_f68 = Some(double(input, profile)?);
        }
        let mut state = self
            .state_1130
            .ok_or_else(|| unsupported("Person prior state1130 is unknown"))?;
        if flags_b & 0x40 != 0 {
            state |= 0x200000;
        }
        state = (state & !0x8000) | (u32::from(flags_b & 0x80 != 0) << 15);
        self.state_1130 = Some(state);
        if flags_c & 0x80 != 0 {
            return Err(unsupported(
                "native Person AuItemHash reception is not implemented",
            ));
        }
        if flags_a & 1 != 0 {
            self.field_10e8 = Some(u32::from(input.read_u8()?));
        }
        if flags_c & 0x40 != 0 {
            let packed = input.read_u8()?;
            self.field_83d = Some(packed & 0x0f);
            self.field_83b = Some(packed >> 4);
        }
        Ok(NativePersonReceive {
            flags_a,
            flags_b,
            flags_c,
            timer_restarted,
            gear,
            jobs,
            consumed_bytes: length - input.remaining(),
        })
    }
}

impl NativePersonIdentity {
    fn receive(&mut self, input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<()> {
        self.id_10ec = Some(input.read_u32()?);
        self.string_10f0 = Some(input.read_string()?);
        self.id_10f8 = Some(input.read_u32()?);
        self.string_1100 = Some(input.read_string()?);
        self.globe_1108 = Some(NativeGlobeLlf::read(input, profile)?);
        self.id_1110 = Some(input.read_u32()?);
        self.field_1114 = Some(input.read_u8()? != 0);
        self.string_1118 = Some(input.read_string()?);
        Ok(())
    }
}

fn double(input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<f64> {
    match profile {
        FloatEncoding::SinglePrecision => input.read_f32().map(f64::from),
        FloatEncoding::TypeWidths | FloatEncoding::DoublePrecision => input.read_f64(),
    }
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}
