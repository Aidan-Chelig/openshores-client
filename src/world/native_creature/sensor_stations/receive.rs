use super::*;

impl NativeStationBlipSelection {
    fn replace(
        &mut self,
        kind: u8,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<Option<u8>> {
        // The old selection is destroyed before reading the new full body.
        self.kind = Some(kind);
        self.value = None;
        self.value = NativeSensorBlipFull::read(kind, input, profile)?.map(Box::new);
        Ok(self.value.is_none().then_some(kind))
    }
}

impl NativeStationSensor {
    /// Decode ordered wire fields. Earlier updates survive a later error;
    /// vectors/full bodies commit only once decoded (Rust safety, not Qt parity).
    /// Local SensorsWorking/environment augmentation is not performed here.
    pub fn receive_wire(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<NativeStationBlipWireReceive> {
        let length = input.remaining();
        let flags = input.read_u16()?;
        self.flags = Some(NativeStationSensorFlags {
            fields_14: bits(flags, [0, 1, 2, 3]),
            fields_18: bits(flags, [4, 5]),
            field_1a: bits(flags, [6])[0],
            fields_40: bits(flags, [12, 15, 13, 14]),
        });
        let kind = input.read_u8()?;
        let null_full_factory = self.blip_50.replace(kind, input, profile)?;
        self.vector_20 = Some(vector(input, profile)?);
        self.field_38 = Some(float(input, profile)?);
        self.field_3c = Some(float(input, profile)?);
        self.id_1c = Some(input.read_u32()?);
        let minimal_list = self.list_58.receive(input, profile)?;
        Ok(NativeStationBlipWireReceive {
            null_full_factory,
            minimal_list,
            consumed_bytes: length - input.remaining(),
        })
    }
}

impl NativeStationWeapon {
    /// Wire stage only; capsule power checks and local sensor augmentation must
    /// follow separately. Null full factories consume no body and continue.
    pub fn receive_wire(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<NativeStationBlipWireReceive> {
        let length = input.remaining();
        let flags = input.read_u16()?;
        self.flags = Some(NativeStationWeaponFlags {
            fields_14: bits(flags, [0, 1, 2, 3]),
            field_18: bits(flags, [4])[0],
            field_48: if flags & 0x40 != 0 {
                1
            } else if flags & 0x80 != 0 {
                0xff
            } else {
                0
            },
            fields_3d: bits(flags, [12, 15, 13, 14]),
        });
        self.field_41 = Some(input.read_u8()?);
        let kind = input.read_u8()?;
        let null_full_factory = self.blip_70.replace(kind, input, profile)?;
        self.vector_20 = Some(vector(input, profile)?);
        self.field_38 = Some(float(input, profile)?);
        self.field_3c = Some(input.read_u8()?);
        self.field_4c = Some(float(input, profile)?);
        self.field_50 = Some(input.read_u16()?);
        self.field_a2 = Some(input.read_u16()?);
        self.field_44 = Some(input.read_i32()?);
        self.id_1c = Some(input.read_u32()?);
        let minimal_list = self.list_78.receive(input, profile)?;
        Ok(NativeStationBlipWireReceive {
            null_full_factory,
            minimal_list,
            consumed_bytes: length - input.remaining(),
        })
    }
}

impl NativeStationTransporter {
    /// Bit20 absence retains vector50 but resets field4d; bit10 absence destroys
    /// the selected full blip. This is not complete capsule augmentation.
    pub fn receive_wire(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<NativeStationBlipWireReceive> {
        let length = input.remaining();
        let flags = input.read_u8()?;
        self.field_18 = Some(u32::from(flags & 3));
        self.field_1c = Some(u32::from(input.read_u8()?));
        self.blip_70 = NativeStationBlipSelection::default();
        let null_full_factory = if flags & 0x10 != 0 {
            let kind = input.read_u8()?;
            self.blip_70.replace(kind, input, profile)?
        } else {
            None
        };
        if flags & 0x20 != 0 {
            self.vector_50 = Some(vector(input, profile)?);
        }
        self.field_4d = Some(flags & 0x20 != 0);
        self.vector_20 = Some(vector(input, profile)?);
        self.field_38 = Some(float(input, profile)?);
        let flags = input.read_u16()?;
        self.flags = Some(NativeStationTransporterFlags {
            fields_40: bits(flags, [2, 3]),
            fields_3c: bits(flags, [4, 5, 0, 1]),
            field_42: bits(flags, [6])[0],
            field_4c: bits(flags, [14])[0],
            fields_48: bits(flags, [11, 12, 15, 13]),
        });
        self.field_44 = Some(float(input, profile)?);
        self.field_ae = Some(input.read_u16()?);
        self.id_14 = Some(input.read_u32()?);
        let minimal_list = self.list_78.receive(input, profile)?;
        Ok(NativeStationBlipWireReceive {
            null_full_factory,
            minimal_list,
            consumed_bytes: length - input.remaining(),
        })
    }
}
