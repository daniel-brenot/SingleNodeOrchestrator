/// This file contains a global instance of readonly values that can be used across the application.
///
/// Any fields added to the config struct will inform the application to attempt to load an environment
/// variable from the system by that name.
/// NOTE: The name of the environment variable it loads will be in all uppercase,
/// so 'example_key' becomes 'EXAMPLE_KEY'
use serde::Deserialize;

/// Describes configurations that originate from the applications environment
#[derive(Deserialize)]
pub struct Config {
    /// Directory where the app deployments are stored
    #[serde(default = "default_apps_dir")]
    pub apps_dir: String,
    /// Directory where scheduled job definitions are stored
    #[serde(default = "default_jobs_dir")]
    pub jobs_dir: String,
}

fn default_apps_dir() -> String {
    "apps".to_string()
}

fn default_jobs_dir() -> String {
    "jobs".to_string()
}

lazy_static! {
    pub static ref CONFIG: Config =
        envy::from_env::<Config>().expect("Failed to load config from environment");
}
