plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "dev.crepuscularity.nativeshell"
    compileSdk = 35

    defaultConfig {
        applicationId = "dev.crepuscularity.nativeshell"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlin { jvmToolchain(17) }

    buildFeatures { compose = true }

    sourceSets["main"].jniLibs.srcDir(layout.buildDirectory.dir("rustJniLibs"))
}

val rustCrateDir = layout.projectDirectory.dir("../../rust")
val rustTargetDir = rustCrateDir.dir("target")
val rustAndroidTarget = "aarch64-linux-android"
val rustJniOutputDir = layout.buildDirectory.dir("rustJniLibs/arm64-v8a")

fun registerRustActionsTask(variantName: String, profile: String) {
    val capitalized = variantName.replaceFirstChar { it.uppercase() }
    val sdkDir = providers.environmentVariable("ANDROID_HOME").orElse(providers.environmentVariable("ANDROID_SDK_ROOT"))
    val ndkDir = providers.environmentVariable("ANDROID_NDK_HOME").orElse(
        sdkDir.map { sdk ->
            file("$sdk/ndk").listFiles()?.filter { it.isDirectory }?.maxByOrNull { it.name }?.absolutePath ?: ""
        }
    )
    tasks.register<Exec>("buildRustActions$capitalized") {
        inputs.files(fileTree(rustCrateDir.dir("src")), rustCrateDir.file("Cargo.toml"))
        outputs.file(rustJniOutputDir.map { it.file("libcrepus_mobile_actions.so") })
        doFirst {
            val ndk = ndkDir.orNull.orEmpty()
            require(ndk.isNotBlank()) { "ANDROID_NDK_HOME or ANDROID_HOME/ndk/<version> is required to build Rust mobile actions" }
            val prebuilt = file("$ndk/toolchains/llvm/prebuilt").listFiles()?.firstOrNull { it.isDirectory }?.absolutePath
                ?: error("NDK toolchain prebuilt directory not found under $ndk")
            val clang = file("$prebuilt/bin/aarch64-linux-android26-clang")
            require(clang.exists()) { "Android NDK clang not found at ${clang.absolutePath}" }
            file(rustJniOutputDir).mkdirs()
            environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", clang.absolutePath)
        }
        commandLine(
            "cargo",
            "build",
            "--manifest-path",
            rustCrateDir.file("Cargo.toml").asFile.absolutePath,
            "--target",
            rustAndroidTarget,
        )
        if (profile == "release") {
            args("--release")
        }
        doLast {
            copy {
                from(rustTargetDir.file("$rustAndroidTarget/$profile/libcrepus_mobile_actions.so"))
                into(rustJniOutputDir)
            }
        }
    }
    afterEvaluate {
        tasks.named("pre${capitalized}Build") {
            dependsOn("buildRustActions$capitalized")
        }
    }
}

registerRustActionsTask("debug", "debug")
registerRustActionsTask("release", "release")

dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.10.01"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
    debugImplementation("androidx.compose.ui:ui-tooling")
}
