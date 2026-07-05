use std::process::Command;

use clap::Args;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Args)]
pub struct BuildOptionsArgs {
    #[arg(long, conflicts_with_all = ["dev", "release"])]
    pub debug: bool,
    #[arg(long, conflicts_with_all = ["debug", "release"])]
    pub dev: bool,
    #[arg(long, conflicts_with_all = ["debug", "dev"])]
    pub release: bool,
    #[arg(long = "opt-level", value_name = "LEVEL")]
    pub opt_level: Option<OptLevelArg>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum OptLevelArg {
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

impl BuildOptionsArgs {
    pub(crate) fn into_options(self) -> Result<BuildOptions, String> {
        let mode = if self.release {
            BuildMode::Release
        } else if self.dev {
            BuildMode::Dev
        } else {
            BuildMode::Debug
        };
        let optimization = self
            .opt_level
            .map(OptLevelArg::into_level)
            .unwrap_or_else(|| mode.default_optimization());
        Ok(BuildOptions { mode, optimization })
    }

    pub(crate) fn into_options_or_exit(self) -> BuildOptions {
        self.into_options().unwrap_or_else(|e| crate::ui::error(&e))
    }
}

impl OptLevelArg {
    fn into_level(self) -> OptimizationLevel {
        match self {
            Self::None => OptimizationLevel::None,
            Self::Fast => OptimizationLevel::Fast,
            Self::Size => OptimizationLevel::Size,
            Self::Aggressive => OptimizationLevel::Aggressive,
        }
    }
}

impl BuildOptions {
    pub(crate) fn debug() -> Self {
        Self {
            mode: BuildMode::Debug,
            optimization: OptimizationLevel::None,
        }
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

    pub(crate) fn apply_profile_to_command(&self, cmd: &mut Command) {
        if self.release() {
            cmd.arg("--release");
        }
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
    pub(crate) fn wasm_opt_flag(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Fast => Some("-O2"),
            Self::Size => Some("-Oz"),
            Self::Aggressive => Some("-O3"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_debug_with_no_post_optimization() {
        let opts = BuildOptionsArgs::default().into_options().expect("parse");
        assert_eq!(opts.mode, BuildMode::Debug);
        assert_eq!(opts.optimization, OptimizationLevel::None);
        assert!(!opts.release());
        assert!(!opts.optimize_artifacts());
    }

    #[test]
    fn release_defaults_to_fast_optimization() {
        let opts = BuildOptionsArgs {
            release: true,
            ..Default::default()
        }
        .into_options()
        .expect("parse");
        assert_eq!(opts.mode, BuildMode::Release);
        assert_eq!(opts.optimization, OptimizationLevel::Fast);
        assert!(opts.release());
        assert!(opts.optimize_artifacts());
    }

    #[test]
    fn explicit_optimization_overrides_mode_default() {
        let opts = BuildOptionsArgs {
            release: true,
            opt_level: Some(OptLevelArg::Size),
            ..Default::default()
        }
        .into_options()
        .expect("parse");
        assert_eq!(opts.mode, BuildMode::Release);
        assert_eq!(opts.optimization, OptimizationLevel::Size);
    }
}
