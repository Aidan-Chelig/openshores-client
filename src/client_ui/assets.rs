//! Installation selection and preparation stay off the render thread.
use super::*;
use crate::asset_mount::cache::{PreparedAssets, default_cache, prepare};
use std::sync::mpsc::{Receiver, TryRecvError, channel};

#[derive(Resource, Default)]
pub(super) struct Setup {
    pub(super) revision: u64,
    picker: Option<Mutex<Receiver<Option<std::path::PathBuf>>>>,
    pending: Option<Mutex<Receiver<io::Result<PreparedAssets>>>>,
    pub(super) prepared: Option<PreparedAssets>,
}

pub(super) fn preparation(mut setup: ResMut<Setup>, mut ui: ResMut<Presentation>) {
    if ui.asset_browse && setup.picker.is_none() && setup.pending.is_none() {
        ui.asset_browse = false;
        let path = ui.asset_path.clone();
        let (tx, rx) = channel();
        setup.picker = Some(Mutex::new(rx));
        std::thread::spawn(move || {
            let mut dialog = rfd::FileDialog::new().set_title("Select your Hazeron installation");
            if !path.is_empty() {
                dialog = dialog.set_directory(path);
            }
            let _ = tx.send(dialog.pick_folder());
        });
    }
    let picked = setup
        .picker
        .as_ref()
        .and_then(|rx| match rx.lock().unwrap().try_recv() {
            Ok(path) => Some(path),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(None),
        });
    if let Some(path) = picked {
        setup.picker = None;
        if let Some(path) = path {
            if let Some(path) = path.to_str() {
                ui.asset_path = path.to_owned();
                ui.asset_request = true;
            } else {
                ui.asset_status = "Installation path must be Unicode".into();
            }
        } else {
            ui.asset_status =
                "Folder selection cancelled or unavailable; enter the path manually.".into();
        }
        ui.dirty = true;
    }
    if ui.asset_request && setup.pending.is_none() {
        setup.revision += 1;
        setup.prepared = None;
        ui.asset_request = false;
        ui.asset_status = "Extracting and converting assets…".into();
        ui.assets_ready = false;
        ui.dirty = true;
        let installation = std::path::PathBuf::from(&ui.asset_path);
        let cache = std::env::var_os("OPENSHORES_ASSET_CACHE")
            .map(std::path::PathBuf::from)
            .map(Ok)
            .unwrap_or_else(default_cache);
        let (tx, rx) = channel();
        setup.pending = Some(Mutex::new(rx));
        std::thread::spawn(move || {
            let result = cache.and_then(|cache| prepare(installation, cache));
            let _ = tx.send(result);
        });
    }
    let result = setup
        .pending
        .as_ref()
        .and_then(|rx| match rx.lock().unwrap().try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                Some(Err(io::Error::other("asset preparation worker stopped")))
            }
        });
    if let Some(result) = result {
        setup.pending = None;
        match result {
            Ok(prepared) => {
                ui.asset_status = format!(
                    "Mounted {} assets • {} geometry conversions • {} opaque models",
                    prepared.mount.entries().len(),
                    prepared.converted_models,
                    prepared.opaque_models
                );
                println!("{}; cache {}", ui.asset_status, prepared.cache.display());
                setup.prepared = Some(prepared);
                ui.assets_ready = true;
            }
            Err(error) => ui.asset_status = format!("Asset preparation failed: {error}"),
        }
        ui.dirty = true;
    }
}
