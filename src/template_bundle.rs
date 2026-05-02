//! Pack and unpack template `.tl` files as `.tpz` (ZIP) bundles.

use std::fs::{self, File};
use std::io::{self, Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};

use chrono::Local;
use glob::glob;
use nu_ansi_term::Color::Yellow;
use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::{ZipArchive, ZipWriter};

use crate::constants::template::DEFAULT_TEMPLATE_EXT;
use crate::error::{Error, Result};

fn warn_skip_existing(dest: &Path) {
    let msg = format!(
        "WARN: skipping {} — file already exists (use --force to overwrite)",
        dest.display()
    );
    let _ = writeln!(io::stderr(), "{}", Yellow.paint(msg));
}

/// `path` must be a descendant of `root` (same logic as [`path_relative_to`]).
fn is_inside_root(root: &Path, path: &Path) -> bool {
    path_relative_to(root, path).is_ok()
}

/// Strip `base` prefix from `path`; both must share a prefix of components.
fn path_relative_to(base: &Path, path: &Path) -> Result<PathBuf> {
    let bc: Vec<Component<'_>> = base.components().collect();
    let pc: Vec<Component<'_>> = path.components().collect();
    if pc.len() < bc.len() || pc[..bc.len()] != bc[..] {
        return Err(Error::Msg(format!(
            "path {} is not under {}",
            path.display(),
            base.display()
        )));
    }
    Ok(pc[bc.len()..].iter().collect())
}

fn arc_path_for_zip(rel: &Path) -> String {
    rel.to_string_lossy().replace('\\', "/")
}

fn sanitize_hostname(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .collect();
    if s.chars().all(|c| c == '_') || s.is_empty() {
        "unknown_host".to_string()
    } else {
        s
    }
}

/// Default bundle basename: `titular_templates_<host>_<timestamp>.tpz`.
#[must_use]
pub fn default_export_basename() -> String {
    let host = sanitize_hostname(
        whoami::fallible::hostname()
            .unwrap_or_else(|_| "unknown".to_string())
            .as_str(),
    );
    let ts = Local::now().format("%Y%m%d_%H%M%S");
    format!("titular_templates_{host}_{ts}.tpz")
}

/// Writes `titular_templates_<host>_<timestamp>.tpz` into the current working directory.
pub fn default_export_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir().map_err(Error::Io)?;
    Ok(cwd.join(default_export_basename()))
}

fn zip_options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
}

/// All `**/*.{DEFAULT_TEMPLATE_EXT}` files under `root`, paths stored relative to `root`.
pub fn export_templates_dir(root: &Path, out: &Path) -> Result<()> {
    if !root.exists() {
        return Err(Error::Msg(format!(
            "templates directory {} does not exist",
            root.display()
        )));
    }

    let pattern = format!(
        "{}{}{}",
        root.to_string_lossy(),
        "/**/*",
        DEFAULT_TEMPLATE_EXT
    );

    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in glob(&pattern).map_err(|e| Error::Msg(format!("Invalid glob pattern: {e}")))? {
        let path = entry.map_err(|e| Error::Msg(format!("Glob iteration error: {e}")))?;
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    let file = File::create(out).map_err(Error::Io)?;
    let mut zip = ZipWriter::new(file);

    for path in paths {
        let rel = path_relative_to(root, &path)?;
        let name = arc_path_for_zip(&rel);
        zip.start_file(name, zip_options())
            .map_err(|e| Error::Msg(format!("zip start_file: {e}")))?;
        let mut f = File::open(&path).map_err(Error::Io)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(Error::Io)?;
        zip.write_all(&buf)
            .map_err(|e| Error::Msg(format!("zip write: {e}")))?;
    }

    zip.finish()
        .map_err(|e| Error::Msg(format!("zip finish: {e}")))?;
    Ok(())
}

fn is_tl_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("tl"))
}

/// Extract `.tl` entries from `archive` into `root`. Unsafe paths are skipped.
pub fn import_bundle_to_templates_dir(archive: &Path, root: &Path, force: bool) -> Result<()> {
    let data = fs::read(archive).map_err(Error::Io)?;
    let reader = Cursor::new(data);
    let mut zip = ZipArchive::new(reader).map_err(zip_err_to_msg)?;

    fs::create_dir_all(root).map_err(Error::Io)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(zip_err_to_msg)?;

        if file.is_dir() {
            continue;
        }

        let Some(rel) = file.enclosed_name() else {
            continue;
        };

        if !is_tl_file(&rel) {
            continue;
        }

        let dest = root.join(&rel);
        if !is_inside_root(root, &dest) {
            return Err(Error::Msg(format!(
                "refusing unsafe zip path: {}",
                rel.display()
            )));
        }

        if dest.exists() && !force {
            warn_skip_existing(&dest);
            continue;
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(Error::Io)?;
        }

        let mut out = File::create(&dest).map_err(Error::Io)?;
        io::copy(&mut file, &mut out).map_err(Error::Io)?;
    }

    Ok(())
}

fn zip_err_to_msg(e: ZipError) -> Error {
    Error::Msg(format!("zip: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_import_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.tl"), b"[pattern]\ndata=x").unwrap();
        let sub = src.join("nested");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("b.tl"), b"[pattern]\ndata=y").unwrap();

        let tpz = tmp.path().join("bundle.tpz");
        export_templates_dir(&src, &tpz).unwrap();

        let dst = tmp.path().join("dst");
        import_bundle_to_templates_dir(&tpz, &dst, false).unwrap();

        assert_eq!(
            fs::read_to_string(dst.join("a.tl")).unwrap(),
            "[pattern]\ndata=x"
        );
        assert_eq!(
            fs::read_to_string(dst.join("nested/b.tl")).unwrap(),
            "[pattern]\ndata=y"
        );
    }

    #[test]
    fn import_skips_existing_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("keep.tl"), b"NEW").unwrap();

        let tpz = tmp.path().join("bundle.tpz");
        export_templates_dir(&src, &tpz).unwrap();

        let dst = tmp.path().join("dst");
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("keep.tl"), b"OLD").unwrap();

        import_bundle_to_templates_dir(&tpz, &dst, false).unwrap();
        assert_eq!(fs::read_to_string(dst.join("keep.tl")).unwrap(), "OLD");
    }

    #[test]
    fn import_overwrites_with_force() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("keep.tl"), b"NEW").unwrap();

        let tpz = tmp.path().join("bundle.tpz");
        export_templates_dir(&src, &tpz).unwrap();

        let dst = tmp.path().join("dst");
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("keep.tl"), b"OLD").unwrap();

        import_bundle_to_templates_dir(&tpz, &dst, true).unwrap();
        assert_eq!(fs::read_to_string(dst.join("keep.tl")).unwrap(), "NEW");
    }
}
