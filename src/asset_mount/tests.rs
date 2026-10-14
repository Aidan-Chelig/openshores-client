use super::*;

#[test]
fn explicit_resource_paths_reject_ambiguity_and_never_match_by_basename() {
    let fixture = Fixture::new();
    let mut mount = AssetMount {
        root: fixture.0.canonicalize().unwrap(),
        entries: BTreeMap::new(),
        backing: vec![b"abcabcxyz".to_vec()],
    };
    for (module, range) in [("A", 0..3), ("B", 3..6)] {
        mount
            .insert(
                format!("qt/{module}/0/0/1/data/model.3ds"),
                MountedAsset {
                    bytes: 3,
                    source: Source::Qt {
                        backing: 0,
                        range,
                        compressed: false,
                    },
                },
            )
            .unwrap();
    }
    assert_eq!(mount.read_resource(":data/model.3ds").unwrap(), b"abc");
    assert_eq!(mount.read_resource(":/data/model.3ds").unwrap(), b"abc");
    assert!(mount.read_resource(":model.3ds").is_err());
    assert!(mount.read_resource(":data/../model.3ds").is_err());
    mount
        .insert(
            "qt/C/0/0/1/data/model.3ds".into(),
            MountedAsset {
                bytes: 3,
                source: Source::Qt {
                    backing: 0,
                    range: 6..9,
                    compressed: false,
                },
            },
        )
        .unwrap();
    assert!(mount.read_resource(":data/model.3ds").is_err());
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "openshores-mount-unit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn loose_assets_are_bounded_and_do_not_include_executables() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("test.png"), b"synthetic texture").unwrap();
    fs::write(fixture.0.join("test.exe"), b"not an asset").unwrap();
    let mut mount = AssetMount {
        root: fixture.0.canonicalize().unwrap(),
        entries: BTreeMap::new(),
        backing: Vec::new(),
    };
    mount.loose(&fixture.0, 0).unwrap();
    assert_eq!(mount.entries().len(), 1);
    assert_eq!(mount.read("loose/test.png").unwrap(), b"synthetic texture");
    assert!(mount.read("loose/../test.png").is_err());
    fs::write(fixture.0.join("test.png"), b"changed").unwrap();
    assert!(mount.read("loose/test.png").is_err());
}

#[cfg(unix)]
#[test]
fn symlink_escape_is_rejected_on_reads_and_skipped_during_indexing() {
    let fixture = Fixture::new();
    let root = fixture.0.join("installation");
    fs::create_dir(&root).unwrap();
    fs::write(fixture.0.join("outside"), b"secret").unwrap();
    std::os::unix::fs::symlink(fixture.0.join("outside"), root.join("link")).unwrap();
    let mut mount = AssetMount {
        root: root.canonicalize().unwrap(),
        entries: BTreeMap::new(),
        backing: Vec::new(),
    };
    mount.loose(&root, 0).unwrap();
    assert!(mount.entries().is_empty());
    mount
        .insert(
            "loose/link".into(),
            MountedAsset {
                bytes: 6,
                source: Source::Loose(root.join("link")),
            },
        )
        .unwrap();
    assert!(mount.read("loose/link").is_err());
}
