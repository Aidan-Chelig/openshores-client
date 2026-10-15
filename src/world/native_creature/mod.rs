//! Ordered Creature receive stages and verified wire components.
//! Concrete context/application and native object dispatch remain required.
use crate::protocol::QtReader;
use std::io;
mod console;
pub use console::{NativeStationGame, NativeStationShield};
mod station_controls;
pub use station_controls::{NativeStationEngineer, NativeStationHelm};
mod surgery;
pub use surgery::NativeStationSurgery;
mod navigation;
pub use navigation::{NativeStationNav, NativeWaypoint, NativeWaypointRoute};
mod power;
pub use power::{NativePowerGroup01, NativePowerGroup02, NativeStationPower};
mod sensor_stations;
pub use sensor_stations::{
    NativeStationBlipSelection, NativeStationBlipWireReceive, NativeStationSensor,
    NativeStationSensorFlags, NativeStationTransporter, NativeStationTransporterFlags,
    NativeStationWeapon, NativeStationWeaponFlags,
};
mod receiving;
pub use receiving::{
    NativeCreatureBodyKind, NativeCreatureBodyReceiveState, NativeCreatureReceive,
    NativeCreatureReceiveContext, NativeCreatureReceiveState,
};
mod pose_application;
pub use pose_application::{
    NativeAnimaliaPoseChange, NativeAnimaliaPoseState, NativeCreaturePoseMarkers,
    NativeDpBodyPoseState, NativeStationPoseRequest, native_animalia_initialize_poses,
    native_current_general_pose, native_station_body_pose, native_station_occupant_position,
    native_station_operator_pose,
};
mod console_state;
mod station_construction;
pub use console_state::{NativeConsoleBody, NativeConsoleState, NativeConsoleWireReceive};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeCreaturePose {
    pub fields: [u32; 10],
    pub station: u16,
}

impl NativeCreaturePose {
    /// Both body branches read ten byte overloads widened unsigned, then one
    /// short. This does not apply SetStationPose/body Pose or restart timers.
    /// Failure is atomic as Rust safety, not Qt malformed-stream parity.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let mut fields = [0; 10];
        for field in &mut fields {
            *field = u32::from(record.read_u8()?);
        }
        let station = record.read_u16()?;
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self { fields, station })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pose_is_ten_widened_bytes_then_short_and_not_native_memory_size() {
        for byte in 0..=255_u8 {
            for station in [0_u16, 0x7fff, 0x8000, 0xffff] {
                let mut bytes: Vec<_> = (0..10).map(|offset| byte.wrapping_add(offset)).collect();
                let expected: Vec<_> = bytes.iter().map(|value| u32::from(*value)).collect();
                bytes.extend(station.to_be_bytes());
                bytes.push(0xde);
                let mut input = QtReader::new(&bytes);
                let pose = NativeCreaturePose::read(&mut input).unwrap();
                assert_eq!(pose.fields.as_slice(), expected);
                assert_eq!(pose.station, station);
                assert_eq!(input.read_u8().unwrap(), 0xde);
                for end in 0..12 {
                    let mut short = QtReader::new(&bytes[..end]);
                    assert_eq!(
                        NativeCreaturePose::read(&mut short).unwrap_err().kind(),
                        io::ErrorKind::UnexpectedEof
                    );
                    assert_eq!(short.remaining(), end);
                }
            }
        }
    }
}
