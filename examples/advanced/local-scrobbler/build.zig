const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    _ = target;
    _ = optimize;
    const root = b.build_root.path orelse ".";
    const cargo_manifest = b.pathJoin(&.{ root, "Cargo.toml" });
    const target_debug = b.pathJoin(&.{ root, "target", "debug" });
    const main_zig = b.pathJoin(&.{ root, "src", "main.zig" });
    const out_bin = b.pathJoin(&.{ root, "zig-out", "bin", "local-scrobbler" });

    const cargo_cmd = b.fmt("SDKROOT=\"${{SDKROOT:-$(xcrun --show-sdk-path)}}\" cargo build --manifest-path {s}", .{cargo_manifest});
    const cargo = b.addSystemCommand(&.{ "sh", "-c", cargo_cmd });
    const make_bin_dir = b.addSystemCommand(&.{ "mkdir", "-p", b.pathJoin(&.{ root, "zig-out", "bin" }) });
    const exe = b.addSystemCommand(&.{
        "zig",
        "build-exe",
        main_zig,
        "-lc",
        b.fmt("-L{s}", .{target_debug}),
        "-llocal_scrobbler",
        "-rpath",
        target_debug,
        b.fmt("-femit-bin={s}", .{out_bin}),
    });
    make_bin_dir.step.dependOn(&cargo.step);
    exe.step.dependOn(&make_bin_dir.step);
    b.getInstallStep().dependOn(&exe.step);

    const test_step = b.step("test", "Build the Zig app and Rust Crepus library");
    test_step.dependOn(&exe.step);
}
