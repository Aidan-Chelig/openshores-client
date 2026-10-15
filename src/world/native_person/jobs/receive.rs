use super::*;

impl NativeJob {
    /// Caller already consumed the factory byte. All27 factory identities are
    /// known and have real readers. This entry uses TypeWidths; the explicit
    /// profile variant handles the BountyHunter double overload.
    /// Fresh-value atomicity on errors is Rust safety, not Qt stream parity.
    pub fn read_qt7(kind: u8, input: &mut QtReader<'_>) -> io::Result<Self> {
        Self::read_qt7_with_profile(kind, input, FloatEncoding::TypeWidths)
    }

    pub fn read_qt7_with_profile(
        kind: u8,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<Self> {
        if kind > 26 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("native job body {kind} is not implemented"),
            ));
        }
        let mut record = QtReader::new(input.remaining_bytes());
        let time_08 = utc_datetime_qt7(&mut record)?;
        let body = match kind {
            1..=3 => {
                NativeJobBody::BountyHunter(NativeJobBountyHunter::read(&mut record, profile)?)
            }
            4 | 5 => NativeJobBody::ShipCommand(NativeJobShipCommand::read(&mut record)?),
            25 => NativeJobBody::TransferPrisoners(NativeJobTransferPrisoners::read(&mut record)?),
            16..=18 => {
                NativeJobBody::DeliverPassengers(NativeJobPassengerDelivery::read(&mut record)?)
            }
            21 | 23 => {
                NativeJobBody::PickUpPassengers(NativeJobPassengerPickup::read(&mut record)?)
            }
            22 => NativeJobBody::PickUpPassengersOverpopulated {
                pickup: NativeJobPassengerPickup::read(&mut record)?,
                id_48: record.read_u32()?,
            },
            6..=8 | 10..=13 => NativeJobBody::DeliverCargo(NativeJobCargo::read(&mut record)?),
            9 => NativeJobBody::DeliverCargoManufacture {
                cargo: NativeJobCargo::read(&mut record)?,
                field_48: u32::from(record.read_u16()?),
            },
            14 | 15 => NativeJobBody::DeliverFood(NativeJobFood::read(&mut record)?),
            19 => NativeJobBody::DestroyEnemyStation {
                string_10: record.read_string()?,
                id_18: record.read_u32()?,
                string_20: record.read_string()?,
                id_28: record.read_u32()?,
                string_30: record.read_string()?,
            },
            26 => NativeJobBody::VisitSystem {
                id_10: record.read_u32()?,
                field_14: record.read_i32()?,
                string_18: record.read_string()?,
                id_20: record.read_u32()?,
                string_28: record.read_string()?,
            },
            0 => NativeJobBody::Assassin {
                string_10: record.read_string()?,
                id_18: record.read_u32()?,
                string_20: record.read_string()?,
                field_28: record.read_i32()?,
            },
            20 => NativeJobBody::InterdictSystem {
                string_10: record.read_string()?,
                id_1c: record.read_u32()?,
                string_20: record.read_string()?,
                field_18: record.read_i32()?,
            },
            24 => NativeJobBody::Story {
                string_10: record.read_string()?,
                string_18: record.read_string()?,
                ignored_wire_byte: record.read_u8()?,
                field_20: record.read_i32()?,
                string_28: record.read_string()?,
            },
            _ => unreachable!(),
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self {
            kind,
            time_08,
            body,
        })
    }
}

impl NativeJobList {
    /// Native clears before count; uses all16 count bits. Types outside0..26
    /// are verified null factories: warn/stop immediately, not continue/skip.
    /// BountyHunter types1/2/3 are discarded only after receiving their bodies.
    pub fn receive_qt7(&mut self, input: &mut QtReader<'_>) -> io::Result<NativeJobListReceive> {
        self.receive_qt7_with_profile(input, FloatEncoding::TypeWidths)
    }

    pub fn receive_qt7_with_profile(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<NativeJobListReceive> {
        let length = input.remaining();
        self.0.clear();
        let declared_count = input.read_u16()?;
        let mut stop = None;
        let mut obsolete_jobs = vec![];
        for index in 0..declared_count {
            let kind = input.read_u8()?;
            if kind > 26 {
                stop = Some(NativeJobStop {
                    unknown_type: kind,
                    unreceived_count: declared_count - index - 1,
                });
                break;
            }
            let job = NativeJob::read_qt7_with_profile(kind, input, profile)?;
            if matches!(kind, 1..=3) {
                obsolete_jobs.push((index, kind));
            } else {
                self.0.push(job);
            }
        }
        Ok(NativeJobListReceive {
            declared_count,
            obsolete_jobs,
            stop,
            consumed_bytes: length - input.remaining(),
        })
    }
}

/// Qt stream version7 uses u32 Julian day, u32 milliseconds and signed byte
/// private time-spec. Encoded2 maps to Qt::UTC. Native then setTimeSpec(UTC).
/// Other specs/invalid date-times need Qt local-time/invalid behavior and fail
/// explicitly; never interpret them using the reference server's profile.
fn utc_datetime_qt7(input: &mut QtReader<'_>) -> io::Result<i64> {
    let day = input.read_u32()?;
    let milliseconds = input.read_u32()?;
    let spec = input.read_u8()?;
    if spec != 2 || day == 0 || milliseconds >= 86_400_000 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native job datetime requires valid Qt7 UTC encoding2; local/invalid semantics are not implemented",
        ));
    }
    Ok((i64::from(day) - 2_440_588) * 86_400_000 + i64::from(milliseconds))
}
