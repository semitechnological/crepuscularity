const std = @import("std");

pub fn build(b: *std.Build) void {
    const root = b.build_root.path orelse ".";
    const zig_build = b.pathJoin(&.{ root, "zig", "build.zig" });
    const test_step = b.step("test", "Run Zig plugin tests");
    const smoke = b.addSystemCommand(&.{ "zig", "build", "test", "--build-file", zig_build });
    test_step.dependOn(&smoke.step);
}
