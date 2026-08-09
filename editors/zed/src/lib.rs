use zed_extension_api::{self as zed, serde_json};

struct CrepuscularityExtension;

impl CrepuscularityExtension {
    fn lsp_settings(worktree: &zed::Worktree) -> Option<zed::settings::LspSettings> {
        zed::settings::LspSettings::for_worktree("crepus-lsp", worktree).ok()
    }

    fn lsp_path(settings: &Option<zed::settings::LspSettings>, worktree: &zed::Worktree) -> String {
        if let Some(path) = settings.as_ref().and_then(|s| s.binary.as_ref()).and_then(|b| b.path.clone()) {
            return path;
        }
        if let Some(path) = worktree.which("crepus-lsp") {
            return path;
        }
        let root = worktree
            .root_path()
            .trim_end_matches(|c| c == '/' || c == '\\')
            .to_string();
        format!("{root}/target/debug/crepus-lsp")
    }

    fn lsp_args(settings: &Option<zed::settings::LspSettings>) -> Vec<String> {
        let mut args = settings
            .as_ref()
            .and_then(|s| s.binary.as_ref())
            .and_then(|b| b.arguments.clone())
            .unwrap_or_default();
        if !args.iter().any(|a| a == "--stdio") {
            args.push("--stdio".into());
        }
        args
    }

    fn lsp_env(settings: &Option<zed::settings::LspSettings>) -> Vec<(String, String)> {
        settings
            .as_ref()
            .and_then(|s| s.binary.as_ref())
            .and_then(|b| b.env.clone())
            .map_or_else(Vec::new, |env| env.into_iter().collect())
    }
}

impl zed::Extension for CrepuscularityExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let settings = Self::lsp_settings(worktree);
        Ok(zed::Command {
            command: Self::lsp_path(&settings, worktree),
            args: Self::lsp_args(&settings),
            env: Self::lsp_env(&settings),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<serde_json::Value>> {
        Ok(Self::lsp_settings(worktree).and_then(|s| s.initialization_options))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<serde_json::Value>> {
        Ok(Self::lsp_settings(worktree).and_then(|s| s.settings))
    }
}

zed::register_extension!(CrepuscularityExtension);
