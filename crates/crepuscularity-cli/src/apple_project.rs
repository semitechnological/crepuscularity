use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::build_options::BuildOptions;
use crate::cli::AppleCommands;
use crate::ui;

#[derive(Clone, Copy)]
pub(crate) enum Platform {
    Ios,
    Macos,
}

impl Platform {
    fn sdk(self) -> &'static str {
        match self {
            Self::Ios => "iphoneos",
            Self::Macos => "macosx",
        }
    }

    fn deployment_key(self) -> &'static str {
        match self {
            Self::Ios => "IPHONEOS_DEPLOYMENT_TARGET",
            Self::Macos => "MACOSX_DEPLOYMENT_TARGET",
        }
    }

    fn product_type(self) -> &'static str {
        "com.apple.product-type.application"
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Config<'a> {
    pub name: &'a str,
    pub bundle_id: &'a str,
    pub platform: Platform,
    pub deployment_target: &'a str,
    pub package_path: &'a str,
    pub package_product: &'a str,
}

pub(crate) fn generate(root: &Path, config: Config<'_>) -> Result<PathBuf, String> {
    let project = root.join(format!("{}.xcodeproj", config.name));
    fs::create_dir_all(&project).map_err(|e| format!("create {}: {e}", project.display()))?;
    fs::write(project.join("project.pbxproj"), project_file(&config))
        .map_err(|e| format!("write project: {e}"))?;
    let scheme_dir = project.join("xcshareddata/xcschemes");
    fs::create_dir_all(&scheme_dir).map_err(|e| format!("create schemes: {e}"))?;
    fs::write(
        scheme_dir.join(format!("{}.xcscheme", config.name)),
        scheme_file(config.name),
    )
    .map_err(|e| format!("write scheme: {e}"))?;
    Ok(project)
}

pub(crate) fn execute(command: AppleCommands) {
    match command {
        AppleCommands::Generate { dir } => {
            let (root, config) = load(dir);
            generate_configured(&root, &config);
        }
        AppleCommands::Build { build, dir } => {
            let options = build.into_options_or_exit();
            let (root, config) = load(dir);
            let project = generate_configured(&root, &config);
            build_project(&root, &project, &config, options);
        }
    }
}

fn load(dir: Option<PathBuf>) -> (PathBuf, crate::crepus_toml::AppleTomlSection) {
    let start =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut root = fs::canonicalize(&start).unwrap_or(start);
    loop {
        if let Some(config) = crate::crepus_toml::try_load_apple(&root.join("crepus.toml")) {
            return (root, config);
        }
        if !root.pop() {
            ui::error("no crepus.toml [apple] found");
        }
    }
}

fn generate_configured(root: &Path, config: &crate::crepus_toml::AppleTomlSection) -> PathBuf {
    let platform = match config.platform.as_str() {
        "ios" => Platform::Ios,
        "macos" => Platform::Macos,
        other => ui::error(&format!(
            "apple platform must be ios or macos, got {other:?}"
        )),
    };
    generate(
        root,
        Config {
            name: &config.name,
            bundle_id: &config.bundle_id,
            platform,
            deployment_target: &config.deployment_target,
            package_path: &config.package_path,
            package_product: &config.package_product,
        },
    )
    .unwrap_or_else(|e| ui::error(&e))
}

fn build_project(
    root: &Path,
    project: &Path,
    config: &crate::crepus_toml::AppleTomlSection,
    options: BuildOptions,
) {
    let project_name = project
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| ui::error("invalid Apple project name"));
    let sdk = match config.platform.as_str() {
        "ios" => "iphonesimulator",
        "macos" => "macosx",
        _ => unreachable!(),
    };
    let mut command = Command::new("xcodebuild");
    command.current_dir(root).args([
        "-project",
        project_name,
        "-scheme",
        &config.name,
        "-sdk",
        sdk,
        "-configuration",
        if options.release() {
            "Release"
        } else {
            "Debug"
        },
        "CODE_SIGNING_ALLOWED=NO",
        "build",
    ]);
    if config.platform == "ios" {
        command.arg("-destination").arg(
            config
                .destination
                .as_deref()
                .unwrap_or("generic/platform=iOS Simulator"),
        );
    }
    let status = command
        .status()
        .unwrap_or_else(|e| ui::error(&format!("xcodebuild: {e}")));
    if !status.success() {
        ui::error("xcodebuild failed");
    }
}

fn project_file(config: &Config<'_>) -> String {
    let target = config.name;
    let product = format!("{target}.app");
    let device_family = match config.platform {
        Platform::Ios => "\n                TARGETED_DEVICE_FAMILY = \"1,2\";",
        Platform::Macos => "",
    };
    format!(
        r#"// !$*UTF8*$!
{{
    archiveVersion = 1;
    classes = {{}};
    objectVersion = 77;
    objects = {{
        000000000000000000000001 = {{isa = PBXBuildFile; fileRef = 000000000000000000000011; }};
        000000000000000000000002 = {{isa = PBXBuildFile; fileRef = 000000000000000000000012; }};
        000000000000000000000003 = {{isa = PBXBuildFile; productRef = 000000000000000000000021; }};
        000000000000000000000011 = {{isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = App.swift; sourceTree = "<group>"; }};
        000000000000000000000012 = {{isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = ContentView.swift; sourceTree = "<group>"; }};
        000000000000000000000013 = {{isa = PBXFileReference; lastKnownFileType = wrapper.application; path = {product}; sourceTree = BUILT_PRODUCTS_DIR; }};
        000000000000000000000014 = {{isa = PBXFileReference; lastKnownFileType = folder; path = {package_path}; sourceTree = SOURCE_ROOT; }};
        000000000000000000000031 = {{isa = PBXFrameworksBuildPhase; buildActionMask = 2147483647; files = (000000000000000000000003,); runOnlyForDeploymentPostprocessing = 0; }};
        000000000000000000000032 = {{isa = PBXSourcesBuildPhase; buildActionMask = 2147483647; files = (000000000000000000000001,000000000000000000000002,); runOnlyForDeploymentPostprocessing = 0; }};
        000000000000000000000041 = {{isa = PBXGroup; children = (000000000000000000000011,000000000000000000000012,); path = App; sourceTree = "<group>"; }};
        000000000000000000000042 = {{isa = PBXGroup; children = (000000000000000000000014,); name = Packages; sourceTree = "<group>"; }};
        000000000000000000000043 = {{isa = PBXGroup; children = (000000000000000000000013,); name = Products; sourceTree = "<group>"; }};
        000000000000000000000044 = {{isa = PBXGroup; children = (000000000000000000000041,000000000000000000000042,000000000000000000000043,); sourceTree = "<group>"; }};
        000000000000000000000051 = {{isa = PBXNativeTarget; buildConfigurationList = 000000000000000000000061; buildPhases = (000000000000000000000032,000000000000000000000031,); buildRules = (); dependencies = (); name = {target}; packageProductDependencies = (000000000000000000000021,); productName = {target}; productReference = 000000000000000000000013; productType = "{product_type}"; }};
        000000000000000000000071 = {{isa = XCBuildConfiguration; buildSettings = {{ GENERATE_INFOPLIST_FILE = YES; PRODUCT_BUNDLE_IDENTIFIER = {bundle_id}; PRODUCT_NAME = "$(TARGET_NAME)"; SDKROOT = {sdk}; SWIFT_VERSION = 5.9; {deployment_key} = {deployment_target};{device_family} }}; name = Debug; }};
        000000000000000000000072 = {{isa = XCBuildConfiguration; buildSettings = {{ GENERATE_INFOPLIST_FILE = YES; PRODUCT_BUNDLE_IDENTIFIER = {bundle_id}; PRODUCT_NAME = "$(TARGET_NAME)"; SDKROOT = {sdk}; SWIFT_VERSION = 5.9; {deployment_key} = {deployment_target};{device_family} }}; name = Release; }};
        000000000000000000000073 = {{isa = XCBuildConfiguration; buildSettings = {{ SDKROOT = {sdk}; {deployment_key} = {deployment_target}; }}; name = Debug; }};
        000000000000000000000074 = {{isa = XCBuildConfiguration; buildSettings = {{ SDKROOT = {sdk}; {deployment_key} = {deployment_target}; }}; name = Release; }};
        000000000000000000000061 = {{isa = XCConfigurationList; buildConfigurations = (000000000000000000000071,000000000000000000000072,); defaultConfigurationIsVisible = 0; defaultConfigurationName = Debug; }};
        000000000000000000000062 = {{isa = XCConfigurationList; buildConfigurations = (000000000000000000000073,000000000000000000000074,); defaultConfigurationIsVisible = 0; defaultConfigurationName = Debug; }};
        000000000000000000000081 = {{isa = XCLocalSwiftPackageReference; relativePath = {package_path}; }};
        000000000000000000000021 = {{isa = XCSwiftPackageProductDependency; productName = {package_product}; }};
        000000000000000000000091 = {{isa = PBXProject; buildConfigurationList = 000000000000000000000062; compatibilityVersion = "Xcode 16.0"; developmentRegion = en; hasScannedForEncodings = 0; knownRegions = (Base,en,); mainGroup = 000000000000000000000044; packageReferences = (000000000000000000000081,); productRefGroup = 000000000000000000000043; projectDirPath = ""; projectRoot = ""; targets = (000000000000000000000051,); }};
    }};
    rootObject = 000000000000000000000091;
}}
"#,
        product = product,
        package_path = config.package_path,
        target = target,
        product_type = config.platform.product_type(),
        bundle_id = config.bundle_id,
        sdk = config.platform.sdk(),
        deployment_key = config.platform.deployment_key(),
        deployment_target = config.deployment_target,
        device_family = device_family,
        package_product = config.package_product,
    )
}

fn scheme_file(name: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Scheme LastUpgradeVersion="2700" version="1.7">
  <BuildAction parallelizeBuildables="YES" buildImplicitDependencies="YES"><BuildActionEntries><BuildActionEntry buildForTesting="YES" buildForRunning="YES" buildForProfiling="YES" buildForArchiving="YES" buildForAnalyzing="YES"><BuildableReference BuildableIdentifier="primary" BlueprintIdentifier="000000000000000000000051" BuildableName="{name}.app" BlueprintName="{name}" ReferencedContainer="container:{name}.xcodeproj"/></BuildActionEntry></BuildActionEntries></BuildAction>
  <LaunchAction buildConfiguration="Debug" selectedDebuggerIdentifier="Xcode.DebuggerFoundation.Debugger.LLDB" selectedLauncherIdentifier="Xcode.DebuggerFoundation.Launcher.LLDB" launchStyle="0" useCustomWorkingDirectory="NO" ignoresPersistentStateOnLaunch="NO" debugDocumentVersioning="YES" debugServiceExtension="internal" allowLocationSimulation="YES"><BuildableProductRunnable runnableDebuggingMode="0"><BuildableReference BuildableIdentifier="primary" BlueprintIdentifier="000000000000000000000051" BuildableName="{name}.app" BlueprintName="{name}" ReferencedContainer="container:{name}.xcodeproj"/></BuildableProductRunnable></LaunchAction>
</Scheme>
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_stable_ios_project() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = Config {
            name: "ExampleApp",
            bundle_id: "com.example.app",
            platform: Platform::Ios,
            deployment_target: "17.0",
            package_path: "NativeShell",
            package_product: "NativeShell",
        };
        let project = generate(tmp.path(), config).expect("generate");
        let first = fs::read_to_string(project.join("project.pbxproj")).expect("read");
        generate(tmp.path(), config).expect("regenerate");
        assert_eq!(
            first,
            fs::read_to_string(project.join("project.pbxproj")).expect("read")
        );
        assert!(first.contains("PRODUCT_BUNDLE_IDENTIFIER = com.example.app"));
        assert!(first.contains("XCLocalSwiftPackageReference"));
    }
}
