const std = @import("std");

pub const ViewIr = struct {
    version: u32,
    json: []u8,
};

pub fn renderIr(allocator: std.mem.Allocator, path: []const u8) !ViewIr {
    const command = try std.fmt.allocPrint(allocator, "\"${{CREPUS_BIN:-crepus}}\" native ir \"{s}\"", .{path});
    defer allocator.free(command);
    const result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &.{ "/bin/sh", "-c", command },
    });
    defer allocator.free(result.stderr);
    if (result.term.Exited != 0) return error.CrepusFailed;
    const parsed = try std.json.parseFromSlice(std.json.Value, allocator, result.stdout, .{});
    defer parsed.deinit();
    const version = @as(u32, @intCast(parsed.value.object.get("version").?.integer));
    return .{ .version = version, .json = result.stdout };
}

test "renderIr decodes ViewIr" {
    const allocator = std.testing.allocator;
    const ir = try renderIr(allocator, "../fixtures/hello.crepus");
    defer allocator.free(ir.json);
    try std.testing.expectEqual(@as(u32, 2), ir.version);
}
