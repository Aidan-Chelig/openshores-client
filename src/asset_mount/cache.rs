//! User-local, content-addressed cache. Never writes to the installation.
use super::*;
use sha1::{Digest, Sha1};
use std::fmt::Write as _;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub struct PreparedAssets {
    pub mount: AssetMount,
    pub cache: PathBuf,
    pub converted_models: usize,
    pub opaque_models: usize,
}

pub fn default_cache() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("XDG_CACHE_HOME").filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(path).join("openshores-client/assets-v1"));
    }
    std::env::var_os("HOME")
        .filter(|p| !p.is_empty())
        .map(|p| PathBuf::from(p).join(".cache/openshores-client/assets-v1"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "set XDG_CACHE_HOME for the asset cache",
            )
        })
}

fn object(path: &Path, bytes: &[u8]) -> io::Result<()> {
    fn verify(path: &Path, bytes: &[u8]) -> io::Result<()> {
        if !fs::symlink_metadata(path)?.is_file() || read_bounded(path)? != bytes {
            return Err(invalid(
                "existing asset cache object is corrupt; choose a new cache directory",
            ));
        }
        Ok(())
    }
    if fs::symlink_metadata(path).is_ok() {
        return verify(path, bytes);
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| invalid("invalid cache object name"))?;
    let temporary = path.with_file_name(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = file
        .write_all(bytes)
        .and_then(|_| file.sync_all())
        .and_then(|_| match fs::hard_link(&temporary, path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => verify(path, bytes),
            Err(error) => Err(error),
        });
    drop(file);
    // Only this call's temporary object; the final object is never overwritten.
    let cleanup = fs::remove_file(temporary);
    result.and(cleanup)
}

/// Extract byte-exact resources and convert understood geometry to debug OBJ.
/// Raw files stay available: OBJ is not a replacement for materials/animation.
pub fn prepare(
    installation: impl AsRef<Path>,
    cache: impl AsRef<Path>,
) -> io::Result<PreparedAssets> {
    let mount = AssetMount::open(installation)?;
    let requested = cache.as_ref();
    let absolute = if requested.is_absolute() {
        requested.to_owned()
    } else {
        std::env::current_dir()?.join(requested)
    };
    if absolute
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(invalid("cache path must not contain parent components"));
    }
    let mut ancestor = absolute.as_path();
    let mut missing = Vec::new();
    while !ancestor.exists() {
        missing.push(
            ancestor
                .file_name()
                .ok_or_else(|| invalid("invalid cache path"))?
                .to_owned(),
        );
        ancestor = ancestor
            .parent()
            .ok_or_else(|| invalid("invalid cache ancestor"))?;
    }
    let mut cache = ancestor.canonicalize()?;
    for component in missing.into_iter().rev() {
        cache.push(component);
    }
    if cache.starts_with(mount.root()) {
        return Err(invalid(
            "asset cache must be outside the Hazeron installation",
        ));
    }
    fs::create_dir_all(&cache)?;
    let mut converted_models = 0;
    let mut opaque_models = 0;
    let mut manifest = String::from("OpenShores asset cache v1\n");
    writeln!(manifest, "Installation: {:?}", mount.root()).map_err(io::Error::other)?;
    for key in mount.entries().keys() {
        let bytes = mount.read(key)?;
        let digest = format!("{:x}", Sha1::digest(&bytes));
        object(&cache.join(format!("{digest}.raw")), &bytes)?;
        writeln!(manifest, "{digest}\t{key:?}").map_err(io::Error::other)?;
        if let Ok(variant) = crate::model::ModelVariant::from_name(key) {
            match crate::model::Model::decode(&bytes, variant) {
                Ok(model) => {
                    let mut output = Vec::new();
                    crate::model_preview::write_obj(&model, &mut output)?;
                    if output.len() as u64 > MAX_FILE {
                        return Err(invalid("converted model exceeds cache object budget"));
                    }
                    object(&cache.join(format!("{digest}.geometry-v1.obj")), &output)?;
                    converted_models += 1;
                }
                Err(error) => {
                    // Retain unrecognized models losslessly, without guessed repair.
                    opaque_models += 1;
                    writeln!(manifest, "Opaque model {key:?}: {error}")
                        .map_err(io::Error::other)?;
                }
            }
        }
    }
    let digest = format!("{:x}", Sha1::digest(manifest.as_bytes()));
    object(
        &cache.join(format!("{digest}.manifest-v1.txt")),
        manifest.as_bytes(),
    )?;
    Ok(PreparedAssets {
        mount,
        cache,
        converted_models,
        opaque_models,
    })
}
