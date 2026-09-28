//! Debug-only incremental dump of the connected Garmin device, launched with `--dump-device`.
//!
//! The device tree is mirrored under the target directory. Files already present locally with the
//! same size are skipped, and downloads go to a `.part` file first, so an interrupted run can be
//! simply repeated. Map images (`.img`, several GB in total) are never downloaded.

use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use mtp_rs::{ByteRange, MtpDevice, ObjectHandle, Storage};

use crate::utils::constants;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Extension of the Garmin map files, excluded from the dump because of their size.
const MAP_EXTENSION: &str = "img";

#[derive(Default)]
struct Stats {
    downloaded: u64,
    skipped: u64,
    excluded: u64,
    failed: u64,
    bytes: u64,
}

/// Turns a device-provided name into a single safe path component.
fn safe_name(name: &str) -> String {
    match name {
        "" | "." | ".." => "_".to_string(),
        _ => name.replace(['/', '\\'], "_"),
    }
}

async fn download_file(
    storage: &Storage,
    handle: ObjectHandle,
    size: u64,
    dest: &Path,
) -> BoxResult<u64> {
    // The device answers a download of an empty file with an error, so there is nothing to fetch.
    if size == 0 {
        File::create(dest)?;
        return Ok(0);
    }

    let part = PathBuf::from(format!("{}.part", dest.display()));
    let result = async {
        let mut file = File::create(&part)?;
        let mut download = storage.download(handle, ByteRange::Full).await?;
        while let Some(chunk) = download.next_chunk().await {
            file.write_all(&chunk?)?;
        }
        file.flush()?;
        fs::rename(&part, dest)?;
        BoxResult::Ok(download.bytes_received())
    }
    .await;

    if result.is_err() {
        let _ = fs::remove_file(&part);
    }
    result
}

async fn dump_storage(storage: &Storage, root: &Path, stats: &mut Stats) -> BoxResult<()> {
    fs::create_dir_all(root)?;
    let mut pending: Vec<(Option<ObjectHandle>, PathBuf)> = vec![(None, root.to_path_buf())];

    while let Some((parent, dir)) = pending.pop() {
        let objs = match storage.list_objects(parent).await {
            Ok(objs) => objs,
            Err(e) => {
                eprintln!("[fail] listing {}: {e}", dir.display());
                stats.failed += 1;
                continue;
            }
        };

        for obj in objs {
            let path = dir.join(safe_name(&obj.filename));
            if obj.is_folder() {
                fs::create_dir_all(&path)?;
                pending.push((Some(obj.handle), path));
                continue;
            }

            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(MAP_EXTENSION))
            {
                stats.excluded += 1;
                continue;
            }

            if fs::metadata(&path).is_ok_and(|m| m.len() == obj.size) {
                stats.skipped += 1;
                continue;
            }

            println!("[get ] {} ({} bytes)", path.display(), obj.size);
            match download_file(storage, obj.handle, obj.size, &path).await {
                Ok(size) => {
                    stats.downloaded += 1;
                    stats.bytes += size;
                }
                Err(e) => {
                    eprintln!("[fail] {}: {e}", path.display());
                    stats.failed += 1;
                }
            }
        }
    }
    Ok(())
}

async fn dump(out_dir: &Path) -> BoxResult<()> {
    let mut garmin = None;
    for info in MtpDevice::list_devices()? {
        let device = MtpDevice::open_by_location(info.location_id).await?;
        if device.device_info().manufacturer.to_uppercase() == constants::MTP_GARMIN_MANUFACTURER {
            garmin = Some(device);
            break;
        }
        let _ = device.close().await;
    }
    let device = garmin.ok_or("No Garmin MTP device connected")?;
    println!(
        "Dumping {} {} into {}",
        device.device_info().manufacturer,
        device.device_info().model,
        out_dir.display()
    );

    let mut stats = Stats::default();
    let result = async {
        let storages = device.storages().await?;
        let multiple = storages.len() > 1;
        for storage in &storages {
            let root = if multiple {
                out_dir.join(safe_name(&storage.info().description))
            } else {
                out_dir.to_path_buf()
            };
            dump_storage(storage, &root, &mut stats).await?;
        }
        BoxResult::Ok(())
    }
    .await;
    let _ = device.close().await;

    println!(
        "Done: {} downloaded ({} bytes), {} unchanged, {} maps excluded, {} failed",
        stats.downloaded, stats.bytes, stats.skipped, stats.excluded, stats.failed
    );
    result
}

/// Mirrors the connected Garmin device into `out_dir`, downloading only what changed.
pub fn run(out_dir: &Path) -> BoxResult<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(dump(out_dir))
}
