use super::*;

impl NativeCreatureReceiveState {
    /// Enclosing atom/base reception must already be complete. Receive the real
    /// Unit stage first, then Creature in native order. No default context exists:
    /// required model/console effects must be implemented or explicitly fail.
    /// Earlier writes survive later Rust errors; Qt failed-stream behavior is
    /// not emulated. DNA/item helpers retain their atomic Rust body policy.
    pub fn receive_qt7(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
        context: &mut impl NativeCreatureReceiveContext,
    ) -> io::Result<NativeCreatureReceive> {
        let length = input.remaining();
        let unit = self.unit.receive(input, profile)?;
        let flags = input.read_u16()?;
        let kind = if flags & 0x100 != 0 {
            NativeCreatureBodyKind::Animalia
        } else {
            NativeCreatureBodyKind::DpBody
        };
        let created = self.body.as_ref().is_none_or(|body| body.kind != kind);
        context.select_body(kind, created)?;
        if created {
            self.body = Some(NativeCreatureBodyReceiveState::selected(kind));
        }
        if flags & 1 != 0 {
            let packed = input.read_u8()?;
            if kind == NativeCreatureBodyKind::DpBody {
                let configuration = [(packed >> 2) & 1, 1 - (packed & 1)];
                context.configure_dp_body(configuration)?;
                self.body.as_mut().unwrap().dp_configuration = Some(configuration);
            }
            let dna = match kind {
                NativeCreatureBodyKind::DpBody => NativeDna::Dh(NativeDhDna::read(input)?),
                NativeCreatureBodyKind::Animalia => NativeDna::Au(NativeAuDna::read_qt7(input)?),
            };
            context.set_dna(&dna, packed)?;
            let body = self.body.as_mut().unwrap();
            body.received_dna = Some(dna);
            body.received_configuration = Some(packed);
        }
        if flags & 2 != 0 {
            let value = match profile {
                FloatEncoding::DoublePrecision => input.read_f64()? as f32,
                FloatEncoding::TypeWidths | FloatEncoding::SinglePrecision => input.read_f32()?,
            };
            context.set_scalar_1e0(value)?;
            self.scalar_1e0 = Some(value);
        }
        if flags & 4 != 0 {
            let clamped = input.read_i16()?.max(-30);
            context.set_hit_points(kind, clamped)?;
            self.body.as_mut().unwrap().hit_points = clamped;
        }
        let old_console_type = context.old_console_type()?;
        if flags & 8 != 0 {
            context.apply_pose(kind, &NativeCreaturePose::read(input)?, self)?;
        }
        let console_type = if flags & 0x10 != 0 {
            Some(input.read_u8()?)
        } else {
            None
        };
        context.receive_console(console_type, old_console_type, input, profile)?;
        self.field_496 = Some(input.read_u8()?);
        self.field_494 = Some(if flags & 0x20 != 0 {
            input.read_u8()?
        } else {
            0
        });
        self.item_698 = None;
        if flags & 0x80 != 0 {
            let item_type = input.remaining_bytes().first().copied().ok_or_else(|| {
                io::Error::new(io::ErrorKind::UnexpectedEof, "Creature item factory type")
            })?;
            if item_type == 0 {
                input.read_u8()?;
            } else {
                self.item_698 = Some(
                    NativeItemValue::read_with_profile(input, profile)?.ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::Unsupported,
                            format!(
                                "native Creature item factory/body {item_type} is not implemented"
                            ),
                        )
                    })?,
                );
            }
        }
        let now = context.wall_clock_ms()?;
        let milliseconds = if flags & 0x40 != 0 {
            now.wrapping_sub(86_400_000)
        } else {
            now
        };
        context.set_body_timestamp(kind, milliseconds)?;
        self.body.as_mut().unwrap().timestamp_ms = milliseconds;
        if flags & 0x200 != 0 {
            self.field_5b8 = Some(u32::from(input.read_u8()?));
            self.field_5bc = Some(u32::from(input.read_u8()?));
        }
        self.field_495 = Some(context.seat_at_current_location()?);
        Ok(NativeCreatureReceive {
            unit,
            flags,
            consumed_bytes: length - input.remaining(),
        })
    }
}
