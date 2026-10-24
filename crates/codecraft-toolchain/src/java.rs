use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaCompilationConfig {
    pub ecj_jar_path: PathBuf,
    pub d8_jar_path: PathBuf,
    pub android_jar_path: Option<PathBuf>,
    pub source_version: String,
    pub target_version: String,
}

impl Default for JavaCompilationConfig {
    fn default() -> Self {
        Self {
            ecj_jar_path: PathBuf::from("toolchain/ecj.jar"),
            d8_jar_path: PathBuf::from("toolchain/d8.jar"),
            android_jar_path: None,
            source_version: "1.8".to_string(),
            target_version: "1.8".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaBuildStep {
    NotStarted,
    CompilingECJ,
    DexingD8,
    ExecutingDalvik,
    Completed,
    Failed,
}

pub struct JavaPipeline {
    pub config: JavaCompilationConfig,
}

impl JavaPipeline {
    pub fn new(config: JavaCompilationConfig) -> Self {
        Self { config }
    }

    /// Menghasilkan argumen baris perintah untuk ECJ batch compiler
    pub fn generate_ecj_args(
        &self,
        source_file: &Path,
        output_classes_dir: &Path,
        classpath: &[PathBuf],
    ) -> Vec<String> {
        let mut args = vec![
            "-source".to_string(),
            self.config.source_version.clone(),
            "-target".to_string(),
            self.config.target_version.clone(),
            "-encoding".to_string(),
            "UTF-8".to_string(),
            "-d".to_string(),
            output_classes_dir.to_string_lossy().to_string(),
        ];

        if !classpath.is_empty() {
            let cp_str = classpath
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(":");
            args.push("-classpath".to_string());
            args.push(cp_str);
        }

        args.push(source_file.to_string_lossy().to_string());
        args
    }

    /// Menghasilkan argumen baris perintah untuk Android D8 dexer
    pub fn generate_d8_args(
        &self,
        class_files: &[PathBuf],
        output_dex_dir: &Path,
        min_api: u32,
    ) -> Vec<String> {
        let mut args = vec![
            "--output".to_string(),
            output_dex_dir.to_string_lossy().to_string(),
            "--min-api".to_string(),
            min_api.to_string(),
        ];

        if let Some(ref android_jar) = self.config.android_jar_path {
            args.push("--lib".to_string());
            args.push(android_jar.to_string_lossy().to_string());
        }

        for cf in class_files {
            args.push(cf.to_string_lossy().to_string());
        }

        args
    }

    /// Menghasilkan baris perintah eksekusi DalvikVM
    pub fn generate_dalvik_cmd(
        dex_path: &Path,
        main_class: &str,
        program_args: &[String],
    ) -> (String, Vec<String>) {
        let program = "dalvikvm".to_string();
        let mut args = vec![
            "-cp".to_string(),
            dex_path.to_string_lossy().to_string(),
            main_class.to_string(),
        ];
        args.extend_from_slice(program_args);
        (program, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecj_arg_generation() {
        let pipeline = JavaPipeline::new(JavaCompilationConfig::default());
        let source = Path::new("Main.java");
        let out_dir = Path::new("build/classes");
        let args = pipeline.generate_ecj_args(source, out_dir, &[]);

        assert!(args.contains(&"-source".to_string()));
        assert!(args.contains(&"1.8".to_string()));
        assert!(args.contains(&"Main.java".to_string()));
    }

    #[test]
    fn test_d8_arg_generation() {
        let pipeline = JavaPipeline::new(JavaCompilationConfig::default());
        let classes = vec![PathBuf::from("build/classes/Main.class")];
        let out_dex = Path::new("build");
        let args = pipeline.generate_d8_args(&classes, out_dex, 26);

        assert!(args.contains(&"--min-api".to_string()));
        assert!(args.contains(&"26".to_string()));
    }
}
