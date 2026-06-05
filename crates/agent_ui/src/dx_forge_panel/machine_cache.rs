use std::{
    collections::VecDeque,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use super::snapshot::{DxForgeReceiptDrilldown, DxForgeSourceRow};

const MACHINE_CACHE_CACHE_TTL: Duration = Duration::from_secs(5);
const MAX_MACHINE_CACHE_ROOTS: usize = 4;
const MAX_MACHINE_CACHE_FILES: usize = 64;
const MAX_MACHINE_CACHE_DIRECTORIES: usize = 512;
const MAX_MACHINE_CACHE_ENTRIES_PER_DIRECTORY: usize = 256;
const MAX_MACHINE_CACHE_TOTAL_ENTRIES: usize = 4096;
const MACHINE_HEADER_BYTES: usize = 8;
const MACHINE_METADATA_SUFFIX: &str = ".machine.meta.json";
const SERIALIZER_DOCUMENT_MAGIC: &[u8; 4] = b"DXM1";
const TYPED_CACHE_MAGIC: &[u8; 8] = b"DXMCACH1";

static MACHINE_CACHE: OnceLock<Mutex<Option<(Instant, Vec<String>, Vec<DxForgeSourceRow>)>>> =
    OnceLock::new();

pub(super) fn machine_cache_rows(workspace_roots: &[String]) -> Vec<DxForgeSourceRow> {
    let cache = MACHINE_CACHE.get_or_init(|| Mutex::new(None));
    let now = Instant::now();

    if let Ok(mut cache) = cache.lock() {
        if let Some((cached_at, cached_roots, rows)) = cache.as_ref()
            && cached_roots == workspace_roots
            && now.duration_since(*cached_at) <= MACHINE_CACHE_CACHE_TTL
        {
            return rows.clone();
        }

        let rows = scan_machine_cache_rows(workspace_roots);
        *cache = Some((now, workspace_roots.to_vec(), rows.clone()));
        return rows;
    }

    scan_machine_cache_rows(workspace_roots)
}

pub(super) fn invalidate_machine_cache_snapshot_cache() {
    let cache = MACHINE_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut cache) = cache.lock() {
        *cache = None;
    }
}

fn scan_machine_cache_rows(workspace_roots: &[String]) -> Vec<DxForgeSourceRow> {
    workspace_roots
        .iter()
        .take(MAX_MACHINE_CACHE_ROOTS)
        .filter_map(|root| {
            let dx_root = Path::new(root).join(".dx");
            if !dx_root.is_dir() {
                return None;
            }

            let mut summary = MachineCacheSummary::default();
            collect_machine_files(&dx_root, &mut summary);
            (summary.total > 0 || summary.scan_errors > 0 || summary.truncated)
                .then(|| machine_cache_row(root, &dx_root, summary))
        })
        .collect()
}

#[derive(Default)]
struct MachineCacheSummary {
    total: usize,
    serializer_documents: usize,
    typed_caches: usize,
    unknown_caches: usize,
    missing_metadata_sidecars: usize,
    scanned_directories: usize,
    scan_errors: usize,
    truncated: bool,
}

fn collect_machine_files(dx_root: &Path, summary: &mut MachineCacheSummary) {
    let mut pending = VecDeque::from([dx_root.to_path_buf()]);
    let mut total_entries = 0usize;

    while let Some(directory) = pending.pop_front() {
        if summary.total >= MAX_MACHINE_CACHE_FILES
            || summary.scanned_directories >= MAX_MACHINE_CACHE_DIRECTORIES
            || total_entries >= MAX_MACHINE_CACHE_TOTAL_ENTRIES
        {
            summary.truncated = true;
            return;
        }

        let Ok(entries) = std::fs::read_dir(&directory) else {
            summary.scan_errors += 1;
            continue;
        };
        summary.scanned_directories += 1;

        let mut directory_entries = 0usize;
        for entry in entries {
            let Ok(entry) = entry else {
                summary.scan_errors += 1;
                continue;
            };
            directory_entries += 1;
            total_entries += 1;
            if directory_entries > MAX_MACHINE_CACHE_ENTRIES_PER_DIRECTORY
                || total_entries > MAX_MACHINE_CACHE_TOTAL_ENTRIES
            {
                summary.truncated = true;
                return;
            }

            let Ok(file_type) = entry.file_type() else {
                summary.scan_errors += 1;
                continue;
            };
            let path = entry.path();
            if file_type.is_dir() {
                pending.push_back(path);
            } else if file_type.is_file() && is_machine_path(&path) {
                if summary.total >= MAX_MACHINE_CACHE_FILES {
                    summary.truncated = true;
                    return;
                }
                inspect_machine_file(&path, summary);
            }
        }
    }
}

fn is_machine_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("machine"))
}

fn inspect_machine_file(path: &Path, summary: &mut MachineCacheSummary) {
    summary.total += 1;

    match machine_header(path).map(|(bytes, len)| machine_family(&bytes, len)) {
        Some(MachineFamily::SerializerDocument) => summary.serializer_documents += 1,
        Some(MachineFamily::TypedCache) => summary.typed_caches += 1,
        Some(MachineFamily::Unknown) | None => summary.unknown_caches += 1,
    }

    if !metadata_path_for(path).is_file() {
        summary.missing_metadata_sidecars += 1;
    }
}

fn machine_header(path: &Path) -> Option<([u8; MACHINE_HEADER_BYTES], usize)> {
    let mut file = File::open(path).ok()?;
    let mut bytes = [0; MACHINE_HEADER_BYTES];
    let len = file.read(&mut bytes).ok()?;
    Some((bytes, len))
}

fn machine_family(header: &[u8; MACHINE_HEADER_BYTES], len: usize) -> MachineFamily {
    if len >= TYPED_CACHE_MAGIC.len() && &header[..TYPED_CACHE_MAGIC.len()] == TYPED_CACHE_MAGIC {
        MachineFamily::TypedCache
    } else if len >= SERIALIZER_DOCUMENT_MAGIC.len()
        && &header[..SERIALIZER_DOCUMENT_MAGIC.len()] == SERIALIZER_DOCUMENT_MAGIC
    {
        MachineFamily::SerializerDocument
    } else {
        MachineFamily::Unknown
    }
}

fn machine_cache_row(
    workspace_root: &str,
    dx_root: &Path,
    summary: MachineCacheSummary,
) -> DxForgeSourceRow {
    let family = family_detail(&summary);
    let metadata = metadata_detail(&summary);

    DxForgeSourceRow {
        label: "Machine Caches".to_string(),
        detail: format!(
            "{} {} detected · Machine family: {} · freshness unchecked: {}",
            summary.total,
            plural(summary.total, "machine cache", "machine caches"),
            family,
            metadata
        ),
        path: display_path(workspace_root, dx_root),
        open_path: dx_root.display().to_string(),
        receipts: vec![DxForgeReceiptDrilldown {
            label: "Machine family".to_string(),
            detail: family,
        }],
        warnings: warnings(&summary),
    }
}

fn family_detail(summary: &MachineCacheSummary) -> String {
    let mut parts = Vec::new();
    if summary.serializer_documents > 0 {
        parts.push(format!(
            "{} serializer document cache (DXM1)",
            summary.serializer_documents
        ));
    }
    if summary.typed_caches > 0 {
        parts.push(format!("{} typed cache (DXMCACH1)", summary.typed_caches));
    }
    if summary.unknown_caches > 0 {
        parts.push(format!("{} unknown machine cache", summary.unknown_caches));
    }

    if parts.is_empty() {
        "no readable machine cache family".to_string()
    } else {
        parts.join(", ")
    }
}

fn metadata_detail(summary: &MachineCacheSummary) -> String {
    if summary.missing_metadata_sidecars == 0 {
        return "metadata sidecars present".to_string();
    }

    format!(
        "{} metadata {} missing",
        summary.missing_metadata_sidecars,
        plural(
            summary.missing_metadata_sidecars,
            "sidecar is",
            "sidecars are"
        )
    )
}

fn warnings(summary: &MachineCacheSummary) -> Vec<String> {
    let mut warnings = Vec::new();
    if summary.missing_metadata_sidecars > 0 {
        warnings.push(format!(
            "{} metadata sidecar(s) missing; freshness unchecked",
            summary.missing_metadata_sidecars
        ));
    }
    if summary.unknown_caches > 0 {
        warnings.push(format!(
            "{} unknown machine cache(s)",
            summary.unknown_caches
        ));
    }
    if summary.scan_errors > 0 {
        warnings.push(format!(
            "{} machine cache scan error(s)",
            summary.scan_errors
        ));
    }
    if summary.truncated {
        warnings.push(format!(
            "machine cache scan stopped after {} files / {} directories / {} entries",
            MAX_MACHINE_CACHE_FILES, MAX_MACHINE_CACHE_DIRECTORIES, MAX_MACHINE_CACHE_TOTAL_ENTRIES
        ));
    }
    warnings
}

fn metadata_path_for(machine_path: &Path) -> PathBuf {
    let Some(file_name) = machine_path.file_name().and_then(|name| name.to_str()) else {
        return machine_path.with_extension("machine.meta.json");
    };

    if let Some(stem) = file_name.strip_suffix(".machine") {
        return machine_path.with_file_name(format!("{stem}{MACHINE_METADATA_SUFFIX}"));
    }

    machine_path.with_extension("machine.meta.json")
}

fn display_path(workspace_root: &str, path: &Path) -> String {
    let root = PathBuf::from(workspace_root);
    path.strip_prefix(&root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn plural(count: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if count == 1 { singular } else { plural }
}

enum MachineFamily {
    SerializerDocument,
    TypedCache,
    Unknown,
}
