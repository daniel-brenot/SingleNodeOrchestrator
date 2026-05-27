use axum::{routing::get, Json, Router};
use serde::Serialize;
use serde_json::Value;
use tokio::process::Command;

pub fn router() -> Router {
    Router::new().route("/api/system/devices", get(devices))
}

async fn devices() -> Json<DeviceInventoryResponse> {
    Json(DeviceInventoryResponse {
        devices: device_inventory().await,
    })
}

async fn device_inventory() -> Vec<DeviceResponse> {
    #[cfg(target_os = "windows")]
    {
        windows_device_inventory().await
    }

    #[cfg(not(target_os = "windows"))]
    {
        unix_device_inventory().await
    }
}

#[cfg(target_os = "windows")]
async fn windows_device_inventory() -> Vec<DeviceResponse> {
    let script = "Get-PnpDevice | Select-Object Class,FriendlyName,InstanceId,Status,Manufacturer | ConvertTo-Json -Depth 2";
    let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .await
    else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_slice::<Value>(&output.stdout) else {
        return Vec::new();
    };

    let entries = match value {
        Value::Array(entries) => entries,
        Value::Object(_) => vec![value],
        _ => Vec::new(),
    };

    entries
        .into_iter()
        .filter_map(windows_device_response)
        .collect()
}

#[cfg(target_os = "windows")]
fn windows_device_response(value: Value) -> Option<DeviceResponse> {
    let class = json_string(&value, "Class");
    let name = json_string(&value, "FriendlyName");
    let id = json_string(&value, "InstanceId");

    if name.is_empty() && id.is_empty() {
        return None;
    }

    let category = device_category(&class, &id, &name);

    if category == DeviceCategory::Other && !looks_like_iommu_device(&class, &id, &name) {
        return None;
    }

    Some(DeviceResponse {
        id,
        name,
        category,
        status: json_string(&value, "Status"),
        manufacturer: json_string(&value, "Manufacturer"),
        source: "Get-PnpDevice".to_string(),
    })
}

#[cfg(not(target_os = "windows"))]
async fn unix_device_inventory() -> Vec<DeviceResponse> {
    let mut devices = Vec::new();

    devices.extend(
        command_lines("lsusb", &[])
            .await
            .into_iter()
            .enumerate()
            .map(|(index, line)| DeviceResponse {
                id: format!("usb-{index}"),
                name: line,
                category: DeviceCategory::Usb,
                status: String::new(),
                manufacturer: String::new(),
                source: "lsusb".to_string(),
            }),
    );

    devices.extend(
        command_lines("lspci", &[])
            .await
            .into_iter()
            .enumerate()
            .map(|(index, line)| DeviceResponse {
                id: format!("pci-{index}"),
                name: line,
                category: DeviceCategory::Pci,
                status: String::new(),
                manufacturer: String::new(),
                source: "lspci".to_string(),
            }),
    );

    devices.extend(
        command_lines("lsscsi", &[])
            .await
            .into_iter()
            .enumerate()
            .map(|(index, line)| DeviceResponse {
                id: format!("scsi-{index}"),
                name: line,
                category: DeviceCategory::Scsi,
                status: String::new(),
                manufacturer: String::new(),
                source: "lsscsi".to_string(),
            }),
    );

    devices.extend(
        command_lines(
            "sh",
            &[
                "-c",
                "for device in /sys/kernel/iommu_groups/*/devices/*; do [ -e \"$device\" ] && echo \"$(basename $(dirname $(dirname \"$device\"))): $(basename \"$device\")\"; done",
            ],
        )
        .await
        .into_iter()
        .enumerate()
        .map(|(index, line)| DeviceResponse {
            id: format!("iommu-{index}"),
            name: line,
            category: DeviceCategory::Other,
            status: String::new(),
            manufacturer: String::new(),
            source: "sysfs iommu_groups".to_string(),
        }),
    );

    devices
}

#[cfg(not(target_os = "windows"))]
async fn command_lines(command: &str, args: &[&str]) -> Vec<String> {
    let Ok(output) = Command::new(command).args(args).output().await else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(target_os = "windows")]
fn json_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

#[cfg(target_os = "windows")]
fn device_category(class: &str, id: &str, name: &str) -> DeviceCategory {
    let class = class.to_ascii_lowercase();
    let id = id.to_ascii_uppercase();
    let name = name.to_ascii_lowercase();

    if id.starts_with("USB\\") || class == "usb" {
        DeviceCategory::Usb
    } else if id.starts_with("PCI\\") || class == "system" && name.contains("pci") {
        DeviceCategory::Pci
    } else if id.starts_with("SCSI\\") || class.contains("scsi") {
        DeviceCategory::Scsi
    } else {
        DeviceCategory::Other
    }
}

#[cfg(target_os = "windows")]
fn looks_like_iommu_device(class: &str, id: &str, name: &str) -> bool {
    let haystack = format!("{class} {id} {name}").to_ascii_lowercase();
    ["iommu", "vt-d", "amd-vi", "dmar"]
        .iter()
        .any(|needle| haystack.contains(needle))
}

#[derive(Serialize)]
struct DeviceInventoryResponse {
    devices: Vec<DeviceResponse>,
}

#[derive(Serialize)]
struct DeviceResponse {
    id: String,
    name: String,
    category: DeviceCategory,
    status: String,
    manufacturer: String,
    source: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DeviceCategory {
    Usb,
    Pci,
    Scsi,
    Other,
}
