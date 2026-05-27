use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;
use sysinfo::{Disk, DiskKind, Disks};

pub fn router() -> Router {
    Router::new()
        .route("/api/storage/drives", get(storage_drives))
        .route("/api/storage/drives/{id}", get(storage_drive))
}

async fn storage_drives() -> Json<StorageDrivesResponse> {
    Json(StorageDrivesResponse {
        drives: drive_inventory(),
    })
}

async fn storage_drive(Path(id): Path<String>) -> Result<Json<DriveResponse>, StatusCode> {
    drive_inventory()
        .into_iter()
        .find(|drive| drive.id == id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

fn drive_inventory() -> Vec<DriveResponse> {
    let disks = Disks::new_with_refreshed_list();

    disks
        .list()
        .iter()
        .enumerate()
        .map(|(index, disk)| drive_response(index, disk))
        .collect()
}

fn drive_response(index: usize, disk: &Disk) -> DriveResponse {
    let total_space = disk.total_space();
    let free_space = disk.available_space();
    let used_space = total_space.saturating_sub(free_space);

    DriveResponse {
        id: drive_id(index, disk),
        custom_name: None,
        smart_health: "unknown".to_string(),
        last_checked_date: "Never".to_string(),
        temperature_celsius: None,
        warning_temperature_celsius: 50.0,
        danger_temperature_celsius: 60.0,
        used_space_gib: bytes_to_gib(used_space),
        free_space_gib: bytes_to_gib(free_space),
        drive_type: drive_type(disk),
        drive_format: drive_format(disk),
        description: Some(drive_description(disk)),
    }
}

fn drive_id(index: usize, disk: &Disk) -> String {
    let source = disk.mount_point().display().to_string();
    let fallback = disk.name().to_string_lossy();
    let base = if source.trim().is_empty() {
        fallback.as_ref()
    } else {
        source.as_str()
    };
    let normalized = base
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if character == '-' || character == '_' {
                Some(character)
            } else {
                Some('-')
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if normalized.is_empty() {
        format!("drive-{index}")
    } else {
        format!("{normalized}-{index}")
    }
}

fn drive_type(disk: &Disk) -> String {
    match disk.kind() {
        DiskKind::HDD => "HDD",
        DiskKind::SSD => "SSD",
        DiskKind::Unknown(_) => "Unknown",
    }
    .to_string()
}

fn drive_format(disk: &Disk) -> String {
    let filesystem = disk.file_system().to_string_lossy();

    if filesystem.trim().is_empty() {
        "Unknown".to_string()
    } else {
        filesystem.to_string()
    }
}

fn drive_description(disk: &Disk) -> String {
    let mut details = vec![format!("Mounted at {}", disk.mount_point().display())];

    if disk.is_removable() {
        details.push("Removable drive".to_string());
    }

    if disk.is_read_only() {
        details.push("Read-only".to_string());
    }

    details.join(". ")
}

fn bytes_to_gib(bytes: u64) -> f32 {
    bytes as f32 / (1024.0 * 1024.0 * 1024.0)
}

#[derive(Serialize)]
struct StorageDrivesResponse {
    drives: Vec<DriveResponse>,
}

#[derive(Serialize)]
struct DriveResponse {
    id: String,
    custom_name: Option<String>,
    smart_health: String,
    last_checked_date: String,
    temperature_celsius: Option<f32>,
    warning_temperature_celsius: f32,
    danger_temperature_celsius: f32,
    used_space_gib: f32,
    free_space_gib: f32,
    drive_type: String,
    drive_format: String,
    description: Option<String>,
}
