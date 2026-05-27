use crate::config::CONFIG;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path as FsPath, PathBuf},
};

pub fn router() -> Router {
    Router::new()
        .route("/api/jobs", get(jobs).post(create_job))
        .route(
            "/api/jobs/{id}",
            get(job).put(update_job).delete(delete_job),
        )
}

async fn jobs() -> Result<Json<JobsResponse>, JobError> {
    Ok(Json(JobsResponse {
        jobs: job_inventory()?,
    }))
}

async fn job(Path(id): Path<String>) -> Result<Json<JobResponse>, JobError> {
    let job_id = normalized_job_name(&id)?;
    let job_dir = job_dir(&job_id);

    if !job_dir.exists() {
        return Err(JobError::not_found("job not found"));
    }

    read_job_dir(&job_dir).map(Json)
}

async fn create_job(Json(request): Json<JobRequest>) -> Result<Json<JobResponse>, JobError> {
    let job_name = normalized_job_name(&request.name)?;
    let job_dir = job_dir(&job_name);

    if job_dir.exists() {
        return Err(JobError::conflict("job already exists"));
    }

    write_job_files(&job_dir, &request)?;
    read_job_dir(&job_dir).map(Json)
}

async fn update_job(
    Path(id): Path<String>,
    Json(request): Json<JobRequest>,
) -> Result<Json<JobResponse>, JobError> {
    let existing_id = normalized_job_name(&id)?;
    let existing_dir = job_dir(&existing_id);

    if !existing_dir.exists() {
        return Err(JobError::not_found("job not found"));
    }

    let job_name = normalized_job_name(&request.name)?;
    let target_dir = job_dir(&job_name);

    if existing_dir != target_dir {
        if target_dir.exists() {
            return Err(JobError::conflict("target job already exists"));
        }

        fs::rename(&existing_dir, &target_dir).map_err(JobError::internal)?;
    }

    write_job_files(&target_dir, &request)?;
    read_job_dir(&target_dir).map(Json)
}

async fn delete_job(Path(id): Path<String>) -> Result<StatusCode, JobError> {
    let job_id = normalized_job_name(&id)?;
    let job_dir = job_dir(&job_id);

    if !job_dir.exists() {
        return Err(JobError::not_found("job not found"));
    }

    fs::remove_dir_all(job_dir).map_err(JobError::internal)?;

    Ok(StatusCode::NO_CONTENT)
}

fn job_inventory() -> Result<Vec<JobResponse>, JobError> {
    let jobs_dir = jobs_dir();

    if !jobs_dir.exists() {
        fs::create_dir_all(&jobs_dir).map_err(JobError::internal)?;
        return Ok(Vec::new());
    }

    let mut jobs = Vec::new();

    for entry in fs::read_dir(jobs_dir).map_err(JobError::internal)? {
        let entry = entry.map_err(JobError::internal)?;

        if entry.file_type().map_err(JobError::internal)?.is_dir() {
            match read_job_dir(&entry.path()) {
                Ok(job) => jobs.push(job),
                Err(error) => tracing::warn!(%error.message, "failed to parse job directory"),
            }
        }
    }

    jobs.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(jobs)
}

fn read_job_dir(job_dir: &FsPath) -> Result<JobResponse, JobError> {
    let readme = fs::read_to_string(job_dir.join("README.md")).unwrap_or_default();
    let config = read_job_config(&job_dir.join("job.yaml"))?;
    let id = job_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let job_type = config.job_type;
    let source = fs::read_to_string(job_dir.join(source_file_name(&job_type))).unwrap_or_default();
    let requirements = fs::read_to_string(job_dir.join("requirements.txt")).unwrap_or_default();
    let readme_details = parse_readme(&readme);
    let name = if readme_details.title.is_empty() {
        config.name.unwrap_or_else(|| id.clone())
    } else {
        readme_details.title
    };

    Ok(JobResponse {
        id,
        name,
        command: command_for(&job_type, &source, &requirements),
        cron: config.cron,
        description: readme_details.description,
        enabled: config.enabled,
        last_run: "Never".to_string(),
        next_run: if config.enabled {
            "Pending schedule calculation".to_string()
        } else {
            "Disabled".to_string()
        },
        job_type,
        source,
        requirements,
    })
}

fn write_job_files(job_dir: &FsPath, request: &JobRequest) -> Result<(), JobError> {
    fs::create_dir_all(job_dir).map_err(JobError::internal)?;

    for file_name in ["script.py", "script.sh"] {
        let path = job_dir.join(file_name);
        if path.exists() && file_name != source_file_name(&request.job_type) {
            fs::remove_file(path).map_err(JobError::internal)?;
        }
    }

    fs::write(job_dir.join("README.md"), render_readme(request)).map_err(JobError::internal)?;
    fs::write(
        job_dir.join("job.yaml"),
        serde_yaml::to_string(&JobConfig {
            name: Some(request.name.trim().to_string()),
            cron: request.cron.trim().to_string(),
            enabled: request.enabled,
            job_type: request.job_type.clone(),
        })
        .map_err(JobError::internal)?,
    )
    .map_err(JobError::internal)?;
    fs::write(
        job_dir.join(source_file_name(&request.job_type)),
        &request.source,
    )
    .map_err(JobError::internal)?;
    fs::write(job_dir.join("requirements.txt"), &request.requirements)
        .map_err(JobError::internal)?;

    Ok(())
}

fn read_job_config(path: &FsPath) -> Result<JobConfig, JobError> {
    let content = fs::read_to_string(path).map_err(JobError::internal)?;
    serde_yaml::from_str(&content).map_err(JobError::internal)
}

fn render_readme(request: &JobRequest) -> String {
    let mut readme = format!("# {}\n\n", request.name.trim());

    if !request.description.trim().is_empty() {
        readme.push_str(request.description.trim());
        readme.push('\n');
    }

    readme
}

fn parse_readme(content: &str) -> ReadmeDetails {
    let mut title = String::new();
    let mut description_lines = Vec::new();

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("# ") {
            title = value.trim().to_string();
            continue;
        }

        if !line.trim().is_empty() {
            description_lines.push(line.trim().to_string());
        }
    }

    ReadmeDetails {
        title,
        description: description_lines.join("\n"),
    }
}

fn command_for(job_type: &str, source: &str, requirements: &str) -> String {
    let install = if job_type == "python3" && !requirements.trim().is_empty() {
        "python3 -m pip install -r requirements.txt && "
    } else {
        ""
    };
    let runner = if job_type == "python3" {
        "python3 -c"
    } else {
        "bash -c"
    };

    format!("{install}{runner} | {}", source.trim())
}

fn source_file_name(job_type: &str) -> &'static str {
    if job_type == "python3" {
        "script.py"
    } else {
        "script.sh"
    }
}

fn normalized_job_name(name: &str) -> Result<String, JobError> {
    let name = name.trim();

    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return Err(JobError::bad_request("job name is invalid"));
    }

    Ok(name.to_string())
}

fn job_dir(name: &str) -> PathBuf {
    jobs_dir().join(name)
}

fn jobs_dir() -> PathBuf {
    PathBuf::from(&CONFIG.jobs_dir)
}

#[derive(Default)]
struct ReadmeDetails {
    title: String,
    description: String,
}

#[derive(Serialize)]
struct JobsResponse {
    jobs: Vec<JobResponse>,
}

type JobRequest = JobResponse;

#[derive(Deserialize, Serialize)]
struct JobResponse {
    id: String,
    name: String,
    command: String,
    cron: String,
    description: String,
    enabled: bool,
    last_run: String,
    next_run: String,
    job_type: String,
    source: String,
    requirements: String,
}

#[derive(Deserialize, Serialize)]
struct JobConfig {
    name: Option<String>,
    cron: String,
    enabled: bool,
    job_type: String,
}

#[derive(Debug)]
struct JobError {
    status: StatusCode,
    message: String,
}

impl JobError {
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

impl IntoResponse for JobError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}
