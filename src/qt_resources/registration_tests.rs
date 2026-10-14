use crate::qt_resources::QtResources;

fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn relative(bytes: &mut [u8], offset: usize, prefix: &[u8], target: usize, next: usize) {
    bytes[offset..offset + prefix.len()].copy_from_slice(prefix);
    word(
        bytes,
        offset + prefix.len(),
        (target as i32 - next as i32) as u32,
    );
}
fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; 1536];
    bytes[..2].copy_from_slice(b"MZ");
    word(&mut bytes, 0x3c, 64);
    bytes[64..68].copy_from_slice(b"PE\0\0");
    bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
    bytes[70..72].copy_from_slice(&2u16.to_le_bytes());
    bytes[84..86].copy_from_slice(&240u16.to_le_bytes());
    bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
    word(&mut bytes, 88 + 108, 16);
    word(&mut bytes, 88 + 120, 0x2000);
    for (header, rva, raw, length, flags) in [
        (328, 0x1000, 512, 128, 0x20000000),
        (368, 0x2000, 1024, 512, 0x40000000),
    ] {
        word(&mut bytes, header + 12, rva);
        word(&mut bytes, header + 16, length);
        word(&mut bytes, header + 20, raw);
        word(&mut bytes, header + 36, flags);
    }
    word(&mut bytes, 1024, 0x2040); // Import name thunks.
    word(&mut bytes, 1036, 0x2050); // Library identity, not used to guess resources.
    word(&mut bytes, 1040, 0x2080); // IAT.
    bytes[1088..1096].copy_from_slice(&0x20b0u64.to_le_bytes());
    let name = b"?qRegisterResourceData@@YA_NHPEBE00@Z\0";
    bytes[1202..1202 + name.len()].copy_from_slice(name);
    relative(&mut bytes, 512, &[0x4c, 0x8d, 0x0d], 0x2114, 0x1007); // Data at 1300.
    bytes[519] = 0xb9;
    word(&mut bytes, 520, 2);
    relative(&mut bytes, 524, &[0x4c, 0x8d, 0x05], 0x213c, 0x1013); // Names at 1340.
    relative(&mut bytes, 531, &[0x48, 0x8d, 0x15], 0x2178, 0x101a); // Tree at 1400.
    relative(&mut bytes, 538, &[0xe8], 0x1044, 0x101f);
    relative(&mut bytes, 580, &[0xff, 0x25], 0x2080, 0x104a);
    bytes[1300..1304].copy_from_slice(&3u32.to_be_bytes());
    bytes[1304..1307].copy_from_slice(b"abc");
    bytes[1340..1342].copy_from_slice(&4u16.to_be_bytes());
    for (index, unit) in "test".encode_utf16().enumerate() {
        bytes[1346 + index * 2..1348 + index * 2].copy_from_slice(&unit.to_be_bytes());
    }
    bytes[1404..1406].copy_from_slice(&2u16.to_be_bytes());
    bytes[1406..1410].copy_from_slice(&1u32.to_be_bytes());
    bytes[1410..1414].copy_from_slice(&1u32.to_be_bytes());
    bytes[1428..1430].copy_from_slice(&0u16.to_be_bytes());
    bytes
}

#[test]
fn registration_import_and_rip_relative_arrays_are_required() {
    let bytes = fixture();
    let resources = QtResources::open_registered(&bytes).unwrap();
    assert_eq!(resources.pool_count, 1);
    assert_eq!(resources.entries()[0].path, ":/test");
    assert_eq!(resources.read_entry(0).unwrap(), b"abc");
    for (offset, value) in [(520, 4), (1203, b'x'), (538, 0x90)] {
        let mut corrupt = bytes.clone();
        corrupt[offset] = value;
        assert!(QtResources::open_registered(&corrupt).is_err());
    }
    for end in 0..bytes.len() {
        assert!(
            QtResources::open_registered(&bytes[..end]).is_err(),
            "accepted truncation {end}"
        );
    }
}
