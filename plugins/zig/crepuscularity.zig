const std = @import("std");

pub const ViewIr = struct {
    version: u32,
    json: []u8,
};

pub const UiDocument = struct {
    html: []const u8,
};

pub fn renderIr(allocator: std.mem.Allocator, path: []const u8) !ViewIr {
    // ponytail: argv exec, no shell
    const result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &.{ "crepus", "native", "ir", path },
    });
    defer allocator.free(result.stderr);
    if (result.term.Exited != 0) return error.CrepusFailed;
    const parsed = try std.json.parseFromSlice(std.json.Value, allocator, result.stdout, .{});
    defer parsed.deinit();
    const version = @as(u32, @intCast(parsed.value.object.get("version").?.integer));
    return .{ .version = version, .json = result.stdout };
}

pub fn renderHtml(allocator: std.mem.Allocator, path: []const u8) !UiDocument {
    var ir = try renderIr(allocator, path);
    defer allocator.free(ir.json);
    const marker = "\"content\":\"";
    const start = std.mem.indexOf(u8, ir.json, marker) orelse return .{ .html = try allocator.dupe(u8, "<div data-crepus-kind=\"stack\" data-axis=\"column\"></div>") };
    const content_start = start + marker.len;
    const rest = ir.json[content_start..];
    const end = std.mem.indexOfScalar(u8, rest, '"') orelse 0;
    return .{ .html = try std.fmt.allocPrint(allocator, "<div data-crepus-kind=\"stack\" data-axis=\"column\">{s}</div>", .{rest[0..end]}) };
}

test "renderIr decodes ViewIr" {
    const allocator = std.testing.allocator;
    const ir = try renderIr(allocator, "../fixtures/hello.crepus");
    defer allocator.free(ir.json);
    try std.testing.expectEqual(@as(u32, 2), ir.version);
}
