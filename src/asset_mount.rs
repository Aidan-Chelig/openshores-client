//! Read-only assets supplied by the user. Containers are parsed, never loaded
//! as native libraries. Stable qualified keys preserve pools and locale variants.
pub mod cache;
#[cfg(test)]
mod tests;
use crate::{har::HarArchive, qt_resources::QtResources};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    ops::Range,
    path::{Path, PathBuf},
};

const MAX_FILE: u64 = 128 * 1024 * 1024;
const MAX_BACKING: usize = 512 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;

enum Source {
    Loose(PathBuf),
    Qt {
        backing: usize,
        range: Range<usize>,
        compressed: bool,
    },
}
pub struct MountedAsset {
    pub bytes: usize,
    source: Source,
}
pub struct AssetMount {
    root: PathBuf,
    entries: BTreeMap<String, MountedAsset>,
    backing: Vec<Vec<u8>>,
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn read_bounded(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    if file.metadata()?.len() > MAX_FILE {
        return Err(invalid("asset file exceeds input budget"));
    }
    let mut bytes = Vec::new();
    (&mut file).take(MAX_FILE + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(invalid("asset file grew beyond input budget"));
    }
    Ok(bytes)
}

impl AssetMount {
    /// Resolve an explicit Qt path supplied by the server catalog. Byte-identical
    /// aliases are equivalent; conflicting module/pool/locale variants are not.
    pub fn read_resource(&self, resource: &str) -> io::Result<Vec<u8>> {
        let path = resource
            .strip_prefix(':')
            .ok_or_else(|| invalid("expected explicit Qt resource path"))?
            .trim_start_matches('/');
        if path.is_empty()
            || path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
            || path.contains(['\\', '\0'])
        {
            return Err(invalid("invalid Qt resource path"));
        }
        let mut found: Option<Vec<u8>> = None;
        for key in self
            .entries
            .keys()
            .filter(|key| key.starts_with("qt/") && key.splitn(6, '/').nth(5) == Some(path))
        {
            let bytes = self.read(key)?;
            if let Some(previous) = &found {
                if previous != &bytes {
                    return Err(invalid(
                        "ambiguous Qt resource across mounted pools/modules/locales",
                    ));
                }
            } else {
                found = Some(bytes);
            }
        }
        found.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "resource absent from selected Hazeron installation",
            )
        })
    }
    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().canonicalize()?;
        if !root.is_dir() {
            return Err(invalid("Hazeron installation must be a directory"));
        }
        let mut mount = Self {
            root,
            entries: BTreeMap::new(),
            backing: Vec::new(),
        };
        // Required profile for the supplied unpatched Windows installation.
        for module in [
            "AuAtomRes13.dll",
            "AuBodyPartsRes13.dll",
            "AuModelRes13.dll",
            "AuSceneClientRes13.dll",
        ] {
            mount.mount_qt(module)?;
        }
        let mut modules = fs::read_dir(&mount.root)?.collect::<Result<Vec<_>, _>>()?;
        modules.sort_by_key(|e| e.file_name());
        for entry in modules {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("module filename must be Unicode"))?;
            if (name.starts_with("Au") && name.ends_with("13.dll") && !name.contains("Res13"))
                || matches!(name.as_str(), "Shores of Hazeron.exe" | "Hazeron Mail.exe")
            {
                match mount.mount_qt(&name) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
        mount.mount_har("sfx.har")?;
        mount.loose(&mount.root.clone(), 0)?;
        Ok(mount)
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn entries(&self) -> &BTreeMap<String, MountedAsset> {
        &self.entries
    }
    fn resolve(&self, path: &Path) -> io::Result<PathBuf> {
        let path = path.canonicalize()?;
        if !path.starts_with(&self.root) {
            return Err(invalid("asset path escapes installation"));
        }
        Ok(path)
    }
    fn insert(&mut self, key: String, asset: MountedAsset) -> io::Result<()> {
        if self.entries.len() >= MAX_ENTRIES || self.entries.insert(key, asset).is_some() {
            return Err(invalid("asset index budget exceeded or duplicate identity"));
        }
        Ok(())
    }
    fn add_backing(&mut self, bytes: Vec<u8>) -> io::Result<()> {
        if self.backing.iter().map(Vec::len).sum::<usize>() + bytes.len() > MAX_BACKING {
            return Err(invalid("mounted container budget exceeded"));
        }
        self.backing.push(bytes);
        Ok(())
    }
    fn mount_qt(&mut self, module: &str) -> io::Result<()> {
        let path = self.resolve(&self.root.join(module))?;
        let bytes = read_bounded(&path)?;
        let resources = QtResources::open_registered(&bytes)
            .map_err(|e| io::Error::new(e.kind(), format!("{module}: {e}")))?;
        let backing = self.backing.len();
        for entry in resources.entries() {
            let key = format!(
                "qt/{module}/{}/{}/{}/{}",
                entry.pool,
                entry.country,
                entry.language,
                entry.path.trim_start_matches(":/")
            );
            self.insert(
                key,
                MountedAsset {
                    bytes: entry.original_bytes,
                    source: Source::Qt {
                        backing,
                        range: entry.payload_range.clone(),
                        compressed: entry.flags & 1 != 0,
                    },
                },
            )?;
        }
        self.add_backing(bytes)
    }
    fn mount_har(&mut self, module: &str) -> io::Result<()> {
        let bytes = read_bounded(&self.resolve(&self.root.join(module))?)?;
        let archive = HarArchive::open_sequential(&bytes)?;
        let backing = self.backing.len();
        for entry in archive.entries() {
            self.insert(
                format!("har/{module}/{}", entry.name),
                MountedAsset {
                    bytes: entry.original_bytes as usize,
                    source: Source::Qt {
                        backing,
                        range: entry.offset as usize + 4
                            ..entry.offset as usize + 4 + entry.compressed_bytes as usize,
                        compressed: true,
                    },
                },
            )?;
        }
        self.add_backing(bytes)
    }
    fn loose(&mut self, directory: &Path, depth: usize) -> io::Result<()> {
        if depth > 32 {
            return Err(invalid("installation directory depth exceeds budget"));
        }
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                continue;
            }
            let path = self.resolve(&entry.path())?;
            if kind.is_dir() {
                self.loose(&path, depth + 1)?;
            } else if kind.is_file()
                && !matches!(
                    path.extension()
                        .and_then(|e| e.to_str())
                        .map(str::to_ascii_lowercase)
                        .as_deref(),
                    Some("dll" | "exe" | "har")
                )
            {
                let relative = path
                    .strip_prefix(&self.root)
                    .map_err(|_| invalid("invalid loose asset root"))?;
                let key = relative
                    .to_str()
                    .ok_or_else(|| invalid("asset path must be Unicode"))?
                    .replace('\\', "/");
                let bytes = entry.metadata()?.len();
                if bytes > MAX_FILE {
                    return Err(invalid("loose asset exceeds input budget"));
                }
                self.insert(
                    format!("loose/{key}"),
                    MountedAsset {
                        bytes: bytes as usize,
                        source: Source::Loose(path),
                    },
                )?;
            }
        }
        Ok(())
    }
    pub fn read(&self, key: &str) -> io::Result<Vec<u8>> {
        let asset = self
            .entries
            .get(key)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mounted asset not found"))?;
        let bytes = match &asset.source {
            Source::Loose(path) => read_bounded(&self.resolve(path)?)?,
            Source::Qt {
                backing,
                range,
                compressed,
            } => {
                let payload = &self.backing[*backing][range.clone()];
                if *compressed {
                    crate::har::qdecompress(payload, MAX_FILE as usize)?
                } else {
                    payload.to_vec()
                }
            }
        };
        if bytes.len() != asset.bytes {
            return Err(invalid("mounted asset length changed"));
        }
        Ok(bytes)
    }
}
