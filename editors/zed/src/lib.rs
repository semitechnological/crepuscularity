use zed_extension_api as zed;

struct CrepuscularityExtension;

impl zed::Extension for CrepuscularityExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let root = worktree
            .root_path()
            .trim_end_matches(|c| c == '/' || c == '\\')
            .to_string();
        let cmd = worktree.which("crepus-lsp").unwrap_or_else(|| {
            format!("{root}/target/debug/crepus-lsp")
        });
        Ok(zed::Command {
            command: cmd,
            args: vec!["--stdio".into()],
            env: vec![],
        })
    }
}

zed::register_extension!(CrepuscularityExtension);
