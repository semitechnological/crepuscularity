#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuildMode {
    Debug,
    Dev,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OptimizationLevel {
    None,
    Fast,
    Size,
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BuildOptions {
    pub(crate) mode: BuildMode,
    pub(crate) optimization: OptimizationLevel,
}

impl BuildOptions {
    pub(crate) fn debug() -> Self {
        Self {
            mode: BuildMode::Debug,
            optimization: OptimizationLevel::None,
        }
    }

    pub(crate) fn parse(args: &[String]) -> Result<Self, String> {
        let mut mode = None;
        let mut optimization = None;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--debug" => set_mode(&mut mode, BuildMode::Debug)?,
                "--dev" => set_mode(&mut mode, BuildMode::Dev)?,
                "--release" => set_mode(&mut mode, BuildMode::Release)?,
                "--opt-level" => {
                    i += 1;
                    let Some(value) = args.get(i) else {
                        return Err("--opt-level expects none, fast, size, or aggressive".into());
                    };
                    optimization = Some(OptimizationLevel::parse(value)?);
                }
                arg if arg.starts_with("--opt-level=") => {
                    let value = arg.trim_start_matches("--opt-level=");
                    optimization = Some(OptimizationLevel::parse(value)?);
                }
                _ => {}
            }
            i += 1;
        }
        let mode = mode.unwrap_or(BuildMode::Debug);
        let optimization = optimization.unwrap_or_else(|| mode.default_optimization());
        Ok(Self { mode, optimization })
    }

    pub(crate) fn parse_or_exit(args: &[String]) -> Self {
        Self::parse(args).unwrap_or_else(|e| crate::ui::error(&e))
    }

    pub(crate) fn release(self) -> bool {
        self.mode == BuildMode::Release
    }

    pub(crate) fn cargo_profile(self) -> &'static str {
        if self.release() {
            "release"
        } else {
            "debug"
        }
    }

    pub(crate) fn optimize_artifacts(self) -> bool {
        self.optimization != OptimizationLevel::None
    }
}

impl BuildMode {
    fn default_optimization(self) -> OptimizationLevel {
        match self {
            BuildMode::Debug | BuildMode::Dev => OptimizationLevel::None,
            BuildMode::Release => OptimizationLevel::Fast,
        }
    }
}

impl OptimizationLevel {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "none" => Ok(Self::None),
            "fast" => Ok(Self::Fast),
            "size" => Ok(Self::Size),
            "aggressive" => Ok(Self::Aggressive),
            other => Err(format!(
                "unknown --opt-level {other:?}; expected none, fast, size, or aggressive"
            )),
        }
    }

    pub(crate) fn wasm_opt_flag(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Fast => Some("-O2"),
            Self::Size => Some("-Oz"),
            Self::Aggressive => Some("-O3"),
        }
    }
}

pub(crate) fn strip_build_options(args: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--debug" | "--dev" | "--release" => {}
            "--opt-level" => {
                i += 1;
                if args.get(i).is_none() {
                    return Err("--opt-level expects none, fast, size, or aggressive".into());
                }
            }
            arg if arg.starts_with("--opt-level=") => {}
            _ => out.push(args[i].clone()),
        }
        i += 1;
    }
    Ok(out)
}

pub(crate) fn strip_build_options_or_exit(args: &[String]) -> Vec<String> {
    strip_build_options(args).unwrap_or_else(|e| crate::ui::error(&e))
}

fn set_mode(slot: &mut Option<BuildMode>, next: BuildMode) -> Result<(), String> {
    if let Some(existing) = *slot {
        if existing != next {
            return Err("choose only one of --debug, --dev, or --release".into());
        }
    }
    *slot = Some(next);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_debug_with_no_post_optimization() {
        let args: Vec<String> = vec![];
        let opts = BuildOptions::parse(&args).expect("parse");
        assert_eq!(opts.mode, BuildMode::Debug);
        assert_eq!(opts.optimization, OptimizationLevel::None);
        assert!(!opts.release());
        assert!(!opts.optimize_artifacts());
    }

    #[test]
    fn release_defaults_to_fast_optimization() {
        let args = vec!["--release".to_string()];
        let opts = BuildOptions::parse(&args).expect("parse");
        assert_eq!(opts.mode, BuildMode::Release);
        assert_eq!(opts.optimization, OptimizationLevel::Fast);
        assert!(opts.release());
        assert!(opts.optimize_artifacts());
    }

    #[test]
    fn explicit_optimization_overrides_mode_default() {
        let args = vec![
            "--release".to_string(),
            "--opt-level".to_string(),
            "size".to_string(),
        ];
        let opts = BuildOptions::parse(&args).expect("parse");
        assert_eq!(opts.mode, BuildMode::Release);
        assert_eq!(opts.optimization, OptimizationLevel::Size);
    }

    #[test]
    fn conflicting_modes_are_rejected() {
        let args = vec!["--debug".to_string(), "--release".to_string()];
        let err = BuildOptions::parse(&args).expect_err("conflict");
        assert!(err.contains("choose only one"));
    }

    #[test]
    fn recognized_flags_can_be_stripped_for_target_parsers() {
        let args = vec![
            "--release".to_string(),
            "--opt-level=size".to_string(),
            "--target".to_string(),
            "docs".to_string(),
        ];
        let stripped = strip_build_options(&args).expect("strip");
        assert_eq!(stripped, vec!["--target".to_string(), "docs".to_string()]);
    }
}
