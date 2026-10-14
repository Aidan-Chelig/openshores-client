//! Offline packed native position inspection; no login or network path.
use openshores_client::{
    atom_position::{FloatEncoding, PackedPosition, native_rotation_vectors},
    protocol::QtReader,
};

const USAGE: &str = "usage: inspect-atom-position/dump-atom-matrix/dump-atom-rotation-vectors type-widths|single|double HEX_RECORD";

pub(super) fn input(
    args: &[String],
    maximum: usize,
) -> Result<(FloatEncoding, Vec<u8>), Box<dyn std::error::Error>> {
    if args.len() != 3 {
        return Err(USAGE.into());
    }
    let encoding = match args[1].as_str() {
        "type-widths" => FloatEncoding::TypeWidths,
        "single" => FloatEncoding::SinglePrecision,
        "double" => FloatEncoding::DoublePrecision,
        _ => return Err(USAGE.into()),
    };
    let hex = args[2].as_bytes();
    // Maximum: flags + parent + six DoublePrecision fields = 53 bytes.
    if hex.is_empty() || hex.len() > maximum * 2 || !hex.len().is_multiple_of(2) {
        return Err("hex input is empty, odd-length or exceeds its byte budget".into());
    }
    let bytes = hex
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0])
                .to_digit(16)
                .ok_or("invalid hex digit")?;
            let low = char::from(pair[1])
                .to_digit(16)
                .ok_or("invalid hex digit")?;
            Ok((high * 16 + low) as u8)
        })
        .collect::<Result<Vec<_>, &str>>()?;
    Ok((encoding, bytes))
}

fn decode(args: &[String]) -> Result<PackedPosition, Box<dyn std::error::Error>> {
    let (encoding, bytes) = input(args, 53)?;
    let mut input = QtReader::new(&bytes);
    let position = PackedPosition::read(&mut input, encoding)?;
    if input.remaining() != 0 {
        return Err("trailing bytes after packed position record".into());
    }
    Ok(position)
}

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if !matches!(
        args.first().map(String::as_str),
        Some("inspect-atom-position" | "dump-atom-matrix" | "dump-atom-rotation-vectors")
    ) {
        return Ok(false);
    }
    let record = decode(args)?;
    if args[0] == "dump-atom-matrix" {
        println!("{:?}", record.local_matrix()?);
        return Ok(true);
    }
    if args[0] == "dump-atom-rotation-vectors" {
        println!("{:?}", native_rotation_vectors(&record.local_matrix()?)?);
        return Ok(true);
    }
    println!("{}", json(&record));
    Ok(true)
}

pub(super) fn json(record: &PackedPosition) -> String {
    let parent = record
        .parent
        .map(|id| id.to_string())
        .unwrap_or("null".into());
    format!(
        "{{\"flags\":{},\"parent\":{},\"position\":{:?},\"rotation\":{:?}}}",
        record.flags, parent, record.position, record.rotation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_profile_exact_consumption_and_bounded_hex_are_required() {
        for (profile, hex) in [
            ("guess", "00"),
            ("single", ""),
            ("single", "0"),
            ("single", "gg"),
            ("single", "é"),
            ("single", "0000"),
            ("double", "0200000000"),
        ] {
            let args = ["inspect-atom-position", profile, hex].map(str::to_owned);
            assert!(decode(&args).is_err());
        }
        let args = vec![
            "inspect-atom-position".into(),
            "double".into(),
            "00".repeat(54),
        ];
        assert!(decode(&args).is_err());
        let args = ["inspect-atom-position", "single", "01F1234567"].map(str::to_owned);
        assert_eq!(decode(&args).unwrap().parent, Some(0xf1234567));
        assert!(decode(&args[..2]).is_err());
    }
}
