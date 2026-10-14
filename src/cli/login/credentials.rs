use super::*;

pub(in crate::cli) fn password_from_stdin(reader: &mut impl Read) -> io::Result<Zeroizing<String>> {
    let mut bytes = Zeroizing::new(Vec::new());
    reader.take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "password input exceeds 4096 bytes",
        ));
    }
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "password input must be UTF-8"))?;
    if text.contains(['\r', '\n']) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "password input must contain one line",
        ));
    }
    Ok(Zeroizing::new(text.to_owned()))
}
