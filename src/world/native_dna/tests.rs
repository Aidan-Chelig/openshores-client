use super::*;
use crate::protocol::QtWriter;

#[test]
fn dhdna_normalizes_only_two_bytes_and_leaves_following_fields() {
    for byte in 0..=255 {
        let mut writer = QtWriter::default();
        writer.write_bytes(Some(&[byte; 24])).unwrap();
        writer.write_u8(0xde);
        let bytes = writer.into_bytes();
        let mut input = QtReader::new(&bytes);
        let value = NativeDhDna::read(&mut input).unwrap();
        let mut expected = [byte; 24];
        expected[5] &= 0x7f;
        expected[23] = 0;
        assert_eq!(value.0, expected);
        assert_eq!(input.read_u8().unwrap(), 0xde);
        for end in 0..28 {
            let mut truncated = QtReader::new(&bytes[..end]);
            assert!(NativeDhDna::read(&mut truncated).is_err());
            assert_eq!(truncated.remaining(), end);
        }
    }
}

#[test]
fn null_wrong_length_and_truncated_dhdna_fail_without_consumption() {
    for raw in [None, Some(vec![]), Some(vec![1; 23]), Some(vec![2; 25])] {
        let mut writer = QtWriter::default();
        writer.write_bytes(raw.as_deref()).unwrap();
        let bytes = writer.into_bytes();
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            NativeDhDna::read(&mut input).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(input.remaining(), bytes.len());
    }
}

#[test]
fn species_preserves_null_empty_and_raw_non_text_bytes_with_budget() {
    for raw in [
        None,
        Some(vec![]),
        Some(vec![0xff, 0, 0xfe]),
        Some(vec![7; 1_048_576]),
    ] {
        let mut writer = QtWriter::default();
        writer.write_bytes(raw.as_deref()).unwrap();
        writer.write_u8(0xde);
        let bytes = writer.into_bytes();
        let mut input = QtReader::new(&bytes);
        assert_eq!(NativeSpecies::read(&mut input).unwrap().0, raw);
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
    let mut writer = QtWriter::default();
    writer.write_bytes(Some(&vec![0; 1_048_577])).unwrap();
    let bytes = writer.into_bytes();
    let mut input = QtReader::new(&bytes);
    assert_eq!(
        NativeSpecies::read(&mut input).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(input.remaining(), bytes.len());
}
