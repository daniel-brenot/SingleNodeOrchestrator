use crate::config::CONFIG;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path as FsPath, PathBuf},
};

pub fn router() -> Router {
    Router::new()
        .route("/api/apps", get(apps).post(create_app))
        .route(
            "/api/apps/{id}",
            get(app).put(update_app).delete(delete_app),
        )
}

async fn apps() -> Result<Json<AppsResponse>, AppError> {
    Ok(Json(AppsResponse {
        apps: app_inventory()?,
    }))
}

async fn app(Path(id): Path<String>) -> Result<Json<AppResponse>, AppError> {
    app_inventory()?
        .into_iter()
        .find(|app| app.id == id)
        .map(Json)
        .ok_or(AppError::not_found("app not found"))
}

async fn create_app(Json(request): Json<AppRequest>) -> Result<Json<AppResponse>, AppError> {
    let app_name = normalized_app_name(&request.name)?;
    let app_dir = app_dir(&app_name);

    if app_dir.exists() {
        return Err(AppError::conflict("app already exists"));
    }

    write_app_files(&app_dir, &request)?;
    read_app_dir(&app_dir).map(Json)
}

async fn update_app(
    Path(id): Path<String>,
    Json(request): Json<AppRequest>,
) -> Result<Json<AppResponse>, AppError> {
    let existing_id = normalized_app_name(&id)?;
    let existing_dir = app_dir(&existing_id);

    if !existing_dir.exists() {
        return Err(AppError::not_found("app not found"));
    }

    let app_name = normalized_app_name(&request.name)?;
    let target_dir = app_dir(&app_name);

    if existing_dir != target_dir {
        if target_dir.exists() {
            return Err(AppError::conflict("target app already exists"));
        }

        fs::rename(&existing_dir, &target_dir).map_err(AppError::internal)?;
    }

    write_app_files(&target_dir, &request)?;
    read_app_dir(&target_dir).map(Json)
}

async fn delete_app(Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let app_id = normalized_app_name(&id)?;
    let app_dir = app_dir(&app_id);

    if !app_dir.exists() {
        return Err(AppError::not_found("app not found"));
    }

    fs::remove_dir_all(app_dir).map_err(AppError::internal)?;

    Ok(StatusCode::NO_CONTENT)
}

fn app_inventory() -> Result<Vec<AppResponse>, AppError> {
    let apps_dir = apps_dir();

    if !apps_dir.exists() {
        fs::create_dir_all(&apps_dir).map_err(AppError::internal)?;
        return Ok(Vec::new());
    }

    let mut apps = Vec::new();

    for entry in fs::read_dir(apps_dir).map_err(AppError::internal)? {
        let entry = entry.map_err(AppError::internal)?;

        if entry.file_type().map_err(AppError::internal)?.is_dir() {
            match read_app_dir(&entry.path()) {
                Ok(app) => apps.push(app),
                Err(error) => tracing::warn!(%error.message, "failed to parse app directory"),
            }
        }
    }

    apps.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(apps)
}

fn read_app_dir(app_dir: &FsPath) -> Result<AppResponse, AppError> {
    let readme = fs::read_to_string(app_dir.join("README.md")).unwrap_or_default();
    let deployment = read_yaml_file(&app_dir.join("deployment.yaml"))?;
    let service = read_yaml_file(&app_dir.join("service.yaml"))?;
    let readme_details = parse_readme(&readme);
    let id = app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let name = value_at(&deployment, &["metadata", "name"]).unwrap_or_else(|| {
        if readme_details.title.is_empty() {
            id.clone()
        } else {
            readme_details.title.clone()
        }
    });
    let namespace = value_at(&deployment, &["metadata", "namespace"])
        .or_else(|| value_at(&service, &["metadata", "namespace"]))
        .unwrap_or_else(|| "default".to_string());
    let container = first_container(&deployment);
    let image = container
        .and_then(|container| value_at(container, &["image"]))
        .unwrap_or_default();
    let mut ports = service_ports(&service);
    let port_descriptions = readme_details.port_descriptions;

    for port in &mut ports {
        let mapping = port.mapping_key();

        if let Some(description) = port_descriptions.get(&mapping) {
            port.description = description.clone();
        }
    }

    let variables = container.map(env_variables).unwrap_or_default();
    let variable_descriptions = readme_details.variable_descriptions;
    let variables = variables
        .into_iter()
        .map(|mut variable| {
            if let Some(description) = variable_descriptions.get(&variable.name) {
                variable.description = description.clone();
            }
            variable
        })
        .collect();

    Ok(AppResponse {
        id,
        namespace,
        name,
        image,
        description: readme_details.description,
        status: "Unknown".to_string(),
        autostart: false,
        replicas: value_at(&deployment, &["spec", "replicas"])
            .and_then(|replicas| replicas.parse::<i32>().ok())
            .unwrap_or(1),
        available_replicas: 0,
        ports,
        variables,
        resources: container.map(app_resources).unwrap_or_default(),
        liveness_probe: container
            .and_then(|container| value_at_value(container, &["livenessProbe"]))
            .map(probe_from_yaml),
        readiness_probe: container
            .and_then(|container| value_at_value(container, &["readinessProbe"]))
            .map(probe_from_yaml),
    })
}

fn write_app_files(app_dir: &FsPath, request: &AppRequest) -> Result<(), AppError> {
    fs::create_dir_all(app_dir).map_err(AppError::internal)?;

    fs::write(app_dir.join("README.md"), render_readme(request)).map_err(AppError::internal)?;
    fs::write(
        app_dir.join("deployment.yaml"),
        serde_yaml::to_string(&deployment_yaml(request)).map_err(AppError::internal)?,
    )
    .map_err(AppError::internal)?;
    fs::write(
        app_dir.join("service.yaml"),
        serde_yaml::to_string(&service_yaml(request)).map_err(AppError::internal)?,
    )
    .map_err(AppError::internal)?;

    Ok(())
}

fn render_readme(request: &AppRequest) -> String {
    let mut readme = format!("# {}\n\n", request.name.trim());

    if !request.description.trim().is_empty() {
        readme.push_str(request.description.trim());
        readme.push_str("\n\n");
    }

    readme.push_str("## Ports\n\n");
    readme.push_str("| Mapping | Description |\n");
    readme.push_str("| --- | --- |\n");

    for port in &request.ports {
        readme.push_str(&format!(
            "| {} | {} |\n",
            port.mapping_key(),
            markdown_cell(&port.description)
        ));
    }

    readme.push_str("\n## Environment Variables\n\n");
    readme.push_str("| Variable | Type | Source | Value | Description |\n");
    readme.push_str("| --- | --- | --- | --- | --- |\n");

    for variable in &request.variables {
        readme.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            markdown_cell(&variable.name),
            markdown_cell(&variable.variable_type),
            markdown_cell(&variable.scope),
            markdown_cell(variable.value_or_reference()),
            markdown_cell(&variable.description)
        ));
    }

    readme
}

fn deployment_yaml(request: &AppRequest) -> Value {
    let app_name = kubernetes_name(&request.name);
    let mut container = mapping([
        ("name", Value::String(app_name.clone())),
        ("image", Value::String(request.image.clone())),
    ]);

    if !request.variables.is_empty() {
        container.insert(
            Value::String("env".to_string()),
            Value::Sequence(
                request
                    .variables
                    .iter()
                    .map(|variable| {
                        Value::Mapping(mapping([
                            ("name", Value::String(variable.name.clone())),
                            (
                                "value",
                                Value::String(variable.value_or_reference().to_string()),
                            ),
                        ]))
                    })
                    .collect(),
            ),
        );
    }

    if !request.ports.is_empty() {
        container.insert(
            Value::String("ports".to_string()),
            Value::Sequence(
                request
                    .ports
                    .iter()
                    .map(|port| {
                        Value::Mapping(mapping([
                            ("containerPort", Value::Number(port.container_port.into())),
                            ("protocol", Value::String(port.protocol.clone())),
                        ]))
                    })
                    .collect(),
            ),
        );
    }

    let resources = resources_yaml(&request.resources);

    if !resources.is_empty() {
        container.insert(
            Value::String("resources".to_string()),
            Value::Mapping(resources),
        );
    }

    if let Some(probe) = request.liveness_probe.as_ref().and_then(probe_yaml) {
        container.insert(Value::String("livenessProbe".to_string()), probe);
    }

    if let Some(probe) = request.readiness_probe.as_ref().and_then(probe_yaml) {
        container.insert(Value::String("readinessProbe".to_string()), probe);
    }

    Value::Mapping(mapping([
        ("apiVersion", Value::String("apps/v1".to_string())),
        ("kind", Value::String("Deployment".to_string())),
        (
            "metadata",
            Value::Mapping(mapping([
                ("name", Value::String(app_name.clone())),
                ("namespace", Value::String(request.namespace.clone())),
            ])),
        ),
        (
            "spec",
            Value::Mapping(mapping([
                ("replicas", Value::Number(1.into())),
                (
                    "selector",
                    Value::Mapping(mapping([(
                        "matchLabels",
                        Value::Mapping(mapping([("app", Value::String(app_name.clone()))])),
                    )])),
                ),
                (
                    "template",
                    Value::Mapping(mapping([
                        (
                            "metadata",
                            Value::Mapping(mapping([(
                                "labels",
                                Value::Mapping(mapping([("app", Value::String(app_name.clone()))])),
                            )])),
                        ),
                        (
                            "spec",
                            Value::Mapping(mapping([(
                                "containers",
                                Value::Sequence(vec![Value::Mapping(container)]),
                            )])),
                        ),
                    ])),
                ),
            ])),
        ),
    ]))
}

fn service_yaml(request: &AppRequest) -> Value {
    let app_name = kubernetes_name(&request.name);

    Value::Mapping(mapping([
        ("apiVersion", Value::String("v1".to_string())),
        ("kind", Value::String("Service".to_string())),
        (
            "metadata",
            Value::Mapping(mapping([
                ("name", Value::String(app_name.clone())),
                ("namespace", Value::String(request.namespace.clone())),
            ])),
        ),
        (
            "spec",
            Value::Mapping(mapping([
                (
                    "selector",
                    Value::Mapping(mapping([("app", Value::String(app_name.clone()))])),
                ),
                (
                    "ports",
                    Value::Sequence(
                        request
                            .ports
                            .iter()
                            .map(|port| {
                                let container_port = port.container_port;
                                let service_port = port.host_port.unwrap_or(container_port);

                                Value::Mapping(mapping([
                                    ("name", Value::String(format!("port-{container_port}"))),
                                    ("port", Value::Number(service_port.into())),
                                    ("targetPort", Value::Number(container_port.into())),
                                    ("protocol", Value::String(port.protocol.clone())),
                                ]))
                            })
                            .collect(),
                    ),
                ),
            ])),
        ),
    ]))
}

fn resources_yaml(resources: &AppResourcesResponse) -> Mapping {
    let mut resources_yaml = Mapping::new();
    let mut requests = Mapping::new();
    let mut limits = Mapping::new();

    insert_non_empty(&mut requests, "cpu", &resources.cpu_request);
    insert_non_empty(&mut requests, "memory", &resources.memory_request);
    insert_non_empty(&mut limits, "cpu", &resources.cpu_limit);
    insert_non_empty(&mut limits, "memory", &resources.memory_limit);

    if !requests.is_empty() {
        resources_yaml.insert(
            Value::String("requests".to_string()),
            Value::Mapping(requests),
        );
    }

    if !limits.is_empty() {
        resources_yaml.insert(Value::String("limits".to_string()), Value::Mapping(limits));
    }

    resources_yaml
}

fn probe_yaml(probe: &ProbeResponse) -> Option<Value> {
    if probe.kind == "none" {
        return None;
    }

    let mut probe_yaml = Mapping::new();
    insert_number_if_present(
        &mut probe_yaml,
        "initialDelaySeconds",
        &probe.initial_delay_seconds,
    );
    insert_number_if_present(&mut probe_yaml, "periodSeconds", &probe.period_seconds);
    insert_number_if_present(&mut probe_yaml, "timeoutSeconds", &probe.timeout_seconds);
    insert_number_if_present(
        &mut probe_yaml,
        "successThreshold",
        &probe.success_threshold,
    );
    insert_number_if_present(
        &mut probe_yaml,
        "failureThreshold",
        &probe.failure_threshold,
    );
    insert_number_if_present(
        &mut probe_yaml,
        "terminationGracePeriodSeconds",
        &probe.termination_grace_period_seconds,
    );

    match probe.kind.as_str() {
        "httpGet" => {
            let mut http_get = mapping([
                ("path", Value::String(probe.http_path.clone())),
                ("scheme", Value::String(probe.http_scheme.clone())),
            ]);
            insert_non_empty(&mut http_get, "host", &probe.http_host);
            insert_port(&mut http_get, "port", &probe.http_port);

            if !probe.http_headers.is_empty() {
                http_get.insert(
                    Value::String("httpHeaders".to_string()),
                    Value::Sequence(
                        probe
                            .http_headers
                            .iter()
                            .map(|header| {
                                Value::Mapping(mapping([
                                    ("name", Value::String(header.name.clone())),
                                    ("value", Value::String(header.value.clone())),
                                ]))
                            })
                            .collect(),
                    ),
                );
            }

            probe_yaml.insert(
                Value::String("httpGet".to_string()),
                Value::Mapping(http_get),
            );
        }
        "tcpSocket" => {
            let mut tcp_socket = Mapping::new();
            insert_non_empty(&mut tcp_socket, "host", &probe.tcp_host);
            insert_port(&mut tcp_socket, "port", &probe.tcp_port);
            probe_yaml.insert(
                Value::String("tcpSocket".to_string()),
                Value::Mapping(tcp_socket),
            );
        }
        "exec" => {
            probe_yaml.insert(
                Value::String("exec".to_string()),
                Value::Mapping(mapping([(
                    "command",
                    Value::Sequence(
                        probe
                            .exec_command
                            .lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty())
                            .map(|line| Value::String(line.to_string()))
                            .collect(),
                    ),
                )])),
            );
        }
        "grpc" => {
            let mut grpc = Mapping::new();
            insert_number_if_present(&mut grpc, "port", &probe.grpc_port);
            insert_non_empty(&mut grpc, "service", &probe.grpc_service);
            probe_yaml.insert(Value::String("grpc".to_string()), Value::Mapping(grpc));
        }
        _ => {}
    }

    Some(Value::Mapping(probe_yaml))
}

fn read_yaml_file(path: &FsPath) -> Result<Value, AppError> {
    let content = fs::read_to_string(path).map_err(AppError::internal)?;
    serde_yaml::from_str(&content).map_err(AppError::internal)
}

fn parse_readme(content: &str) -> ReadmeDetails {
    let mut details = ReadmeDetails::default();
    let mut section = "";
    let mut description_lines = Vec::new();

    for line in content.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            details.title = title.trim().to_string();
            continue;
        }

        if let Some(title) = line.strip_prefix("## ") {
            section = title.trim();
            continue;
        }

        if line.starts_with('|') {
            let cells: Vec<String> = line
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect();

            if cells
                .iter()
                .all(|cell| cell.chars().all(|ch| ch == '-' || ch == ' '))
            {
                continue;
            }

            match section {
                "Ports" if cells.len() >= 2 && cells[0] != "Mapping" => {
                    details
                        .port_descriptions
                        .insert(cells[0].clone(), cells[1].clone());
                }
                "Environment Variables" if cells.len() >= 5 && cells[0] != "Variable" => {
                    details
                        .variable_descriptions
                        .insert(cells[0].clone(), cells[4].clone());
                }
                _ => {}
            }

            continue;
        }

        if section.is_empty() && !line.trim().is_empty() {
            description_lines.push(line.trim().to_string());
        }
    }

    details.description = description_lines.join("\n");
    details
}

fn first_container(deployment: &Value) -> Option<&Value> {
    value_at_value(deployment, &["spec", "template", "spec", "containers"])
        .and_then(Value::as_sequence)
        .and_then(|containers| containers.first())
}

fn service_ports(service: &Value) -> Vec<AppPortResponse> {
    value_at_value(service, &["spec", "ports"])
        .and_then(Value::as_sequence)
        .map(|ports| {
            ports
                .iter()
                .filter_map(|port| {
                    let container_port = value_at(port, &["targetPort"])
                        .or_else(|| value_at(port, &["port"]))?
                        .parse::<i32>()
                        .ok()?;
                    let host_port = value_at(port, &["port"]).and_then(|port| port.parse().ok());
                    Some(AppPortResponse {
                        container_port,
                        host_port,
                        protocol: value_at(port, &["protocol"])
                            .unwrap_or_else(|| "TCP".to_string()),
                        description: String::new(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn env_variables(container: &Value) -> Vec<AppVariableResponse> {
    value_at_value(container, &["env"])
        .and_then(Value::as_sequence)
        .map(|variables| {
            variables
                .iter()
                .filter_map(|variable| {
                    let name = value_at(variable, &["name"])?;
                    Some(AppVariableResponse {
                        scope: "container".to_string(),
                        name,
                        value: value_at(variable, &["value"]).unwrap_or_default(),
                        variable_type: "string".to_string(),
                        description: String::new(),
                        global_variable_name: String::new(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn app_resources(container: &Value) -> AppResourcesResponse {
    AppResourcesResponse {
        cpu_request: value_at(container, &["resources", "requests", "cpu"]).unwrap_or_default(),
        cpu_limit: value_at(container, &["resources", "limits", "cpu"]).unwrap_or_default(),
        memory_request: value_at(container, &["resources", "requests", "memory"])
            .unwrap_or_default(),
        memory_limit: value_at(container, &["resources", "limits", "memory"]).unwrap_or_default(),
    }
}

fn probe_from_yaml(probe: &Value) -> ProbeResponse {
    let mut response = ProbeResponse {
        initial_delay_seconds: value_at(probe, &["initialDelaySeconds"]).unwrap_or_default(),
        period_seconds: value_at(probe, &["periodSeconds"]).unwrap_or_default(),
        timeout_seconds: value_at(probe, &["timeoutSeconds"]).unwrap_or_default(),
        success_threshold: value_at(probe, &["successThreshold"]).unwrap_or_default(),
        failure_threshold: value_at(probe, &["failureThreshold"]).unwrap_or_default(),
        termination_grace_period_seconds: value_at(probe, &["terminationGracePeriodSeconds"])
            .unwrap_or_default(),
        ..ProbeResponse::default()
    };

    if let Some(http_get) = value_at_value(probe, &["httpGet"]) {
        response.kind = "httpGet".to_string();
        response.http_path = value_at(http_get, &["path"]).unwrap_or_default();
        response.http_port = value_at(http_get, &["port"]).unwrap_or_default();
        response.http_host = value_at(http_get, &["host"]).unwrap_or_default();
        response.http_scheme =
            value_at(http_get, &["scheme"]).unwrap_or_else(|| "HTTP".to_string());
    } else if let Some(tcp_socket) = value_at_value(probe, &["tcpSocket"]) {
        response.kind = "tcpSocket".to_string();
        response.tcp_port = value_at(tcp_socket, &["port"]).unwrap_or_default();
        response.tcp_host = value_at(tcp_socket, &["host"]).unwrap_or_default();
    } else if let Some(exec) = value_at_value(probe, &["exec"]) {
        response.kind = "exec".to_string();
        response.exec_command = value_at_value(exec, &["command"])
            .and_then(Value::as_sequence)
            .map(|command| {
                command
                    .iter()
                    .filter_map(value_to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
    } else if let Some(grpc) = value_at_value(probe, &["grpc"]) {
        response.kind = "grpc".to_string();
        response.grpc_port = value_at(grpc, &["port"]).unwrap_or_default();
        response.grpc_service = value_at(grpc, &["service"]).unwrap_or_default();
    }

    response
}

fn value_at(value: &Value, path: &[&str]) -> Option<String> {
    value_at_value(value, path).and_then(value_to_string)
}

fn value_at_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;

    for segment in path {
        current = current
            .as_mapping()?
            .get(Value::String(segment.to_string()))?;
    }

    Some(current)
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn mapping<const N: usize>(entries: [(&str, Value); N]) -> Mapping {
    let mut mapping = Mapping::new();

    for (key, value) in entries {
        mapping.insert(Value::String(key.to_string()), value);
    }

    mapping
}

fn insert_non_empty(mapping: &mut Mapping, key: &str, value: &str) {
    if !value.trim().is_empty() {
        mapping.insert(
            Value::String(key.to_string()),
            Value::String(value.to_string()),
        );
    }
}

fn insert_number_if_present(mapping: &mut Mapping, key: &str, value: &str) {
    if let Ok(value) = value.parse::<i32>() {
        mapping.insert(Value::String(key.to_string()), Value::Number(value.into()));
    }
}

fn insert_port(mapping: &mut Mapping, key: &str, value: &str) {
    if let Ok(number) = value.parse::<i32>() {
        mapping.insert(Value::String(key.to_string()), Value::Number(number.into()));
    } else if !value.trim().is_empty() {
        mapping.insert(
            Value::String(key.to_string()),
            Value::String(value.to_string()),
        );
    }
}

fn normalized_app_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();

    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return Err(AppError::bad_request("app name is invalid"));
    }

    Ok(name.to_string())
}

fn kubernetes_name(name: &str) -> String {
    let normalized = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if normalized.is_empty() {
        "app".to_string()
    } else {
        normalized
    }
}

fn app_dir(name: &str) -> PathBuf {
    apps_dir().join(name)
}

fn apps_dir() -> PathBuf {
    PathBuf::from(&CONFIG.apps_dir)
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

#[derive(Default)]
struct ReadmeDetails {
    title: String,
    description: String,
    port_descriptions: BTreeMap<String, String>,
    variable_descriptions: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct AppsResponse {
    apps: Vec<AppResponse>,
}

#[derive(Deserialize, Serialize)]
struct AppResponse {
    id: String,
    namespace: String,
    name: String,
    image: String,
    description: String,
    status: String,
    autostart: bool,
    replicas: i32,
    available_replicas: i32,
    ports: Vec<AppPortResponse>,
    variables: Vec<AppVariableResponse>,
    resources: AppResourcesResponse,
    liveness_probe: Option<ProbeResponse>,
    readiness_probe: Option<ProbeResponse>,
}

type AppRequest = AppResponse;

#[derive(Deserialize, Serialize)]
struct AppPortResponse {
    container_port: i32,
    host_port: Option<i32>,
    protocol: String,
    #[serde(default)]
    description: String,
}

impl AppPortResponse {
    fn mapping_key(&self) -> String {
        match self.host_port {
            Some(host_port) => format!("{host_port}->{} {}", self.container_port, self.protocol),
            None => format!("{} {}", self.container_port, self.protocol),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
struct AppVariableResponse {
    scope: String,
    name: String,
    value: String,
    #[serde(rename = "type")]
    variable_type: String,
    description: String,
    global_variable_name: String,
}

impl AppVariableResponse {
    fn value_or_reference(&self) -> &str {
        if self.scope == "global" && !self.global_variable_name.is_empty() {
            &self.global_variable_name
        } else {
            &self.value
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
struct AppResourcesResponse {
    cpu_request: String,
    cpu_limit: String,
    memory_request: String,
    memory_limit: String,
}

#[derive(Default, Deserialize, Serialize)]
struct ProbeResponse {
    kind: String,
    initial_delay_seconds: String,
    period_seconds: String,
    timeout_seconds: String,
    success_threshold: String,
    failure_threshold: String,
    termination_grace_period_seconds: String,
    http_path: String,
    http_port: String,
    http_host: String,
    http_scheme: String,
    #[serde(default)]
    http_headers: Vec<ProbeHeaderResponse>,
    tcp_port: String,
    tcp_host: String,
    exec_command: String,
    grpc_port: String,
    grpc_service: String,
}

#[derive(Default, Deserialize, Serialize)]
struct ProbeHeaderResponse {
    name: String,
    value: String,
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn internal(error: impl std::fmt::Display) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: error.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}
