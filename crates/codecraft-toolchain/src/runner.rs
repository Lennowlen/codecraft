use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    pub name: String,
    pub extensions: Vec<String>,
    pub compile: Option<String>,
    pub run: String,
    #[serde(default)]
    pub interactive: bool,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u64,
}

fn default_timeout_secs() -> u64 {
    20
}

fn default_max_memory_mb() -> u64 {
    128
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnersManifest {
    pub runners: HashMap<String, RunnerConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamType {
    Stdout,
    Stderr,
    SystemNotice,
}

#[derive(Debug, Clone)]
pub struct ExecutionStreamChunk {
    pub stream_type: StreamType,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub success: bool,
    pub total_duration_ms: u128,
    pub memory_peak_kb: Option<u64>,
}

pub struct RunnerManager {
    runners: HashMap<String, RunnerConfig>,
}

impl Default for RunnerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RunnerManager {
    pub fn new() -> Self {
        let mut manager = Self {
            runners: HashMap::new(),
        };
        manager.register_default_runners();
        manager
    }

    pub fn register_default_runners(&mut self) {
        // Java Dalvik/ECJ runner
        self.runners.insert(
            "java".to_string(),
            RunnerConfig {
                name: "Java Local Runner (ECJ + D8)".to_string(),
                extensions: vec!["java".to_string()],
                compile: Some("ecj -source 1.8 -target 1.8 -d {build_dir}/classes {file} && d8 --output {build_dir} {build_dir}/classes/*.class".to_string()),
                run: "dalvikvm -cp {build_dir}/classes.dex {main_class}".to_string(),
                interactive: true,
                timeout_secs: 30,
                max_memory_mb: 128,
            },
        );

        // Python runner
        self.runners.insert(
            "python".to_string(),
            RunnerConfig {
                name: "Python 3 Local".to_string(),
                extensions: vec!["py".to_string()],
                compile: None,
                run: "python3 -u {file}".to_string(),
                interactive: true,
                timeout_secs: 20,
                max_memory_mb: 64,
            },
        );

        // C Clang runner
        self.runners.insert(
            "c".to_string(),
            RunnerConfig {
                name: "Clang C (NDK / Termux)".to_string(),
                extensions: vec!["c".to_string()],
                compile: Some("clang -O2 {file} -o {build_dir}/{file_stem}".to_string()),
                run: "{build_dir}/{file_stem}".to_string(),
                interactive: true,
                timeout_secs: 15,
                max_memory_mb: 64,
            },
        );
    }

    pub fn load_manifest(&mut self, toml_str: &str) -> Result<(), toml::de::Error> {
        let manifest: RunnersManifest = toml::from_str(toml_str)?;
        for (k, v) in manifest.runners {
            self.runners.insert(k, v);
        }
        Ok(())
    }

    pub fn get_runner_for_file(&self, path: &Path) -> Option<&RunnerConfig> {
        let ext = path.extension()?.to_str()?;
        self.runners.values().find(|r| r.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)))
    }

    pub fn resolve_command(
        template: &str,
        file: &Path,
        build_dir: &Path,
        main_class: Option<&str>,
    ) -> String {
        let file_str = file.to_string_lossy();
        let file_stem = file.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
        let build_dir_str = build_dir.to_string_lossy();
        let main_class_str = main_class.unwrap_or(&file_stem);

        template
            .replace("{file}", &file_str)
            .replace("{file_stem}", &file_stem)
            .replace("{build_dir}", &build_dir_str)
            .replace("{main_class}", main_class_str)
    }
}
