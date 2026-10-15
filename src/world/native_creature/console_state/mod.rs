//! Typed console station storage and wire binding. Not yet concrete Qt/UI,
//! SitAtConsole lifecycle, client-settings or capsule augmentation effects.
use super::*;
use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::io;
mod receive;

#[derive(Clone, Debug, PartialEq)]
pub enum NativeConsoleBody {
    Empty { field_20_nonzero: bool },
    Command,
    Engineer(NativeStationEngineer),
    Weapon(NativeStationWeapon),
    Helm(NativeStationHelm),
    Surgery(NativeStationSurgery),
    Nav(NativeStationNav),
    Power(NativeStationPower),
    Sensor(NativeStationSensor),
    Shield(NativeStationShield),
    Transporter(NativeStationTransporter),
    Game(NativeStationGame),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeConsoleState {
    /// Class/factory identity, distinct from current +f0 (Game and Empty vary).
    pub factory_type: u8,
    pub body: NativeConsoleBody,
}

impl NativeConsoleState {
    /// Construct only the receive-visible station subset of supported native
    /// console classes. Type0 is the real Empty class used by SitAtConsole;
    /// it is NOT accepted by Creature's separate temporary receive switch.
    /// Unsupported factory types return an explicit error, never a guessed body.
    pub fn from_native_constructor(kind: u8, station: u16) -> io::Result<Self> {
        let body = match kind {
            0 => NativeConsoleBody::Empty {
                field_20_nonzero: false,
            },
            1 | 0x0c => {
                NativeConsoleBody::Engineer(NativeStationEngineer::from_native_constructor(station))
            }
            2 | 0x0d => {
                NativeConsoleBody::Weapon(NativeStationWeapon::from_native_constructor(station))
            }
            3 | 0x0e => NativeConsoleBody::Helm(NativeStationHelm::from_native_constructor()),
            4 | 0x0f => {
                NativeConsoleBody::Surgery(NativeStationSurgery::from_native_constructor(station))
            }
            5 | 0x10 => NativeConsoleBody::Nav(NativeStationNav::from_native_constructor()),
            7 | 0x12 => NativeConsoleBody::Power(NativeStationPower::from_native_constructor()),
            8 | 0x13 => NativeConsoleBody::Sensor(NativeStationSensor::from_native_constructor()),
            9 | 0x14 => NativeConsoleBody::Shield(NativeStationShield::from_native_constructor()),
            10 | 0x15 => NativeConsoleBody::Transporter(
                NativeStationTransporter::from_native_constructor(station),
            ),
            0x0b => NativeConsoleBody::Command,
            0x21..=0x23 => {
                NativeConsoleBody::Game(NativeStationGame::from_native_constructor(kind, station))
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("native console class {kind} is not implemented"),
                ));
            }
        };
        Ok(Self {
            factory_type: kind,
            body,
        })
    }

    /// Constructor-confirmed primary+f0 getter. Empty depends on its native
    /// field20 nonzero predicate; Game widens its current raw byte14.
    pub fn console_type(&self) -> u32 {
        match &self.body {
            NativeConsoleBody::Empty { field_20_nonzero } => {
                if *field_20_nonzero {
                    0x3d
                } else {
                    0
                }
            }
            NativeConsoleBody::Game(value) => u32::from(value.field_14),
            _ => u32::from(self.factory_type),
        }
    }

    /// Constructor-confirmed primary+e0 getter. Helm's short26 and other raw
    /// receive fields are NOT station numbers; their getter returns zero.
    pub fn station_number(&self) -> io::Result<u16> {
        Ok(match &self.body {
            NativeConsoleBody::Engineer(v) => v.field_2a,
            NativeConsoleBody::Surgery(v) => v.field_28,
            NativeConsoleBody::Game(v) => v.field_16,
            NativeConsoleBody::Weapon(v) => v.field_a2.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native Weapon station number is unobserved",
                )
            })?,
            NativeConsoleBody::Transporter(v) => v.field_ae.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native Transporter station number is unobserved",
                )
            })?,
            _ => 0,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "inspect sensor list stops and perform required local augmentation before completing console reception"]
pub struct NativeConsoleWireReceive {
    pub consumed_bytes: usize,
    /// Some means a Sensor/Weapon/Transporter wire stage completed; its real
    /// local capsule/environment augmentation is still required, not skipped.
    pub sensor_stage: Option<NativeStationBlipWireReceive>,
}

#[cfg(test)]
mod metadata_tests;
#[cfg(test)]
mod tests;
