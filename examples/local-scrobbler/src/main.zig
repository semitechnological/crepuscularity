const std = @import("std");
const c = @cImport({
    @cInclude("stdio.h");
    @cInclude("stdlib.h");
    @cInclude("time.h");
});

const CurrentTrackCallback = *const fn (?*anyopaque, c_int, [*]u8, usize) callconv(.c) c_int;
const TrackActionCallback = *const fn (?*anyopaque, c_int, [*]const u8, usize, [*]const u8, usize, [*]const u8, usize, [*]u8, usize) callconv(.c) c_int;

const CrepusScrobblerCallbacks = extern struct {
    user_data: ?*anyopaque,
    current_track: ?CurrentTrackCallback,
    now_playing: ?TrackActionCallback,
    scrobble: ?TrackActionCallback,
};

extern fn crepus_local_scrobbler_run(callbacks: CrepusScrobblerCallbacks) c_int;

const spotify_cmd =
    "osascript -e 'tell application \"Spotify\" to if it is running then artist of current track & \"\t\" & name of current track & \"\t\" & album of current track & \"\t\" & player state as string else \"stopped\"' 2>/dev/null";
const music_cmd =
    "osascript -e 'tell application \"Music\" to if it is running then artist of current track & \"\t\" & name of current track & \"\t\" & album of current track & \"\t\" & player state as string else \"stopped\"' 2>/dev/null";

fn writeOut(out: [*]u8, out_len: usize, value: []const u8) c_int {
    if (out_len == 0) return -1;
    const n = @min(value.len, out_len - 1);
    @memcpy(out[0..n], value[0..n]);
    out[n] = 0;
    return @intCast(n);
}

fn currentTrack(_: ?*anyopaque, source: c_int, out: [*]u8, out_len: usize) callconv(.c) c_int {
    const cmd = if (source == 0) spotify_cmd else music_cmd;
    const pipe = c.popen(cmd, "r") orelse return writeOut(out, out_len, "unavailable");
    defer _ = c.pclose(pipe);

    var buf: [2048]u8 = undefined;
    if (c.fgets(&buf, buf.len, pipe) == null) {
        return writeOut(out, out_len, "stopped");
    }

    var len: usize = 0;
    while (len < buf.len and buf[len] != 0 and buf[len] != '\n' and buf[len] != '\r') {
        len += 1;
    }
    if (len == 0) return writeOut(out, out_len, "stopped");
    return writeOut(out, out_len, buf[0..len]);
}

fn nowPlaying(user_data: ?*anyopaque, source: c_int, artist: [*]const u8, artist_len: usize, title: [*]const u8, title_len: usize, album: [*]const u8, album_len: usize, out: [*]u8, out_len: usize) callconv(.c) c_int {
    _ = user_data;
    _ = source;
    return lastfm("track.updateNowPlaying", artist[0..artist_len], title[0..title_len], album[0..album_len], null, out, out_len);
}

fn scrobble(user_data: ?*anyopaque, source: c_int, artist: [*]const u8, artist_len: usize, title: [*]const u8, title_len: usize, album: [*]const u8, album_len: usize, out: [*]u8, out_len: usize) callconv(.c) c_int {
    _ = user_data;
    _ = source;
    var timestamp_buf: [32]u8 = undefined;
    const timestamp = std.fmt.bufPrint(&timestamp_buf, "{d}", .{c.time(null)}) catch return writeOut(out, out_len, "timestamp formatting failed");
    return lastfm("track.scrobble", artist[0..artist_len], title[0..title_len], album[0..album_len], timestamp, out, out_len);
}

fn env(name: [*:0]const u8) ?[]const u8 {
    const value = c.getenv(name) orelse return null;
    return std.mem.span(value);
}

fn shellQuote(allocator: std.mem.Allocator, value: []const u8) ![]u8 {
    var quoted: std.ArrayList(u8) = .empty;
    try quoted.append(allocator, '\'');
    for (value) |byte| {
        if (byte == '\'') {
            try quoted.appendSlice(allocator, "'\\''");
        } else {
            try quoted.append(allocator, byte);
        }
    }
    try quoted.append(allocator, '\'');
    return quoted.toOwnedSlice(allocator);
}

fn appendCurlParam(cmd: *std.ArrayList(u8), allocator: std.mem.Allocator, key: []const u8, value: []const u8) !void {
    const pair = try std.fmt.allocPrint(allocator, "{s}={s}", .{ key, value });
    const quoted = try shellQuote(allocator, pair);
    try cmd.appendSlice(allocator, " --data-urlencode ");
    try cmd.appendSlice(allocator, quoted);
}

fn signature(allocator: std.mem.Allocator, api_secret: []const u8, params: []const []const u8) ![]u8 {
    var raw: std.ArrayList(u8) = .empty;
    defer raw.deinit(allocator);
    var i: usize = 0;
    while (i < params.len) : (i += 2) {
        try raw.appendSlice(allocator, params[i]);
        try raw.appendSlice(allocator, params[i + 1]);
    }
    try raw.appendSlice(allocator, api_secret);
    var digest: [16]u8 = undefined;
    std.crypto.hash.Md5.hash(raw.items, &digest, .{});
    const hex = std.fmt.bytesToHex(digest, .lower);
    return allocator.dupe(u8, &hex);
}

fn lastfm(method: []const u8, artist: []const u8, title: []const u8, album: []const u8, timestamp: ?[]const u8, out: [*]u8, out_len: usize) c_int {
    const api_key = env("LASTFM_API_KEY") orelse return writeOut(out, out_len, "LASTFM_API_KEY is missing");
    const api_secret = env("LASTFM_API_SECRET") orelse return writeOut(out, out_len, "LASTFM_API_SECRET is missing");
    const session_key = env("LASTFM_SESSION_KEY") orelse return writeOut(out, out_len, "LASTFM_SESSION_KEY is missing");

    var arena = std.heap.ArenaAllocator.init(std.heap.c_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    var signed_params: std.ArrayList([]const u8) = .empty;
    signed_params.appendSlice(allocator, &.{ "album", album, "api_key", api_key, "artist", artist, "method", method, "sk", session_key, "track", title }) catch return writeOut(out, out_len, "parameter allocation failed");
    if (timestamp) |ts| signed_params.appendSlice(allocator, &.{ "timestamp", ts }) catch return writeOut(out, out_len, "parameter allocation failed");
    const api_sig = signature(allocator, api_secret, signed_params.items) catch return writeOut(out, out_len, "signature allocation failed");

    var cmd: std.ArrayList(u8) = .empty;
    cmd.appendSlice(allocator, "curl -fsS -X POST https://ws.audioscrobbler.com/2.0/") catch return writeOut(out, out_len, "curl command allocation failed");
    appendCurlParam(&cmd, allocator, "album", album) catch return writeOut(out, out_len, "curl command allocation failed");
    appendCurlParam(&cmd, allocator, "api_key", api_key) catch return writeOut(out, out_len, "curl command allocation failed");
    appendCurlParam(&cmd, allocator, "artist", artist) catch return writeOut(out, out_len, "curl command allocation failed");
    appendCurlParam(&cmd, allocator, "method", method) catch return writeOut(out, out_len, "curl command allocation failed");
    appendCurlParam(&cmd, allocator, "sk", session_key) catch return writeOut(out, out_len, "curl command allocation failed");
    appendCurlParam(&cmd, allocator, "track", title) catch return writeOut(out, out_len, "curl command allocation failed");
    if (timestamp) |ts| appendCurlParam(&cmd, allocator, "timestamp", ts) catch return writeOut(out, out_len, "curl command allocation failed");
    appendCurlParam(&cmd, allocator, "api_sig", api_sig) catch return writeOut(out, out_len, "curl command allocation failed");
    appendCurlParam(&cmd, allocator, "format", "json") catch return writeOut(out, out_len, "curl command allocation failed");
    cmd.append(allocator, 0) catch return writeOut(out, out_len, "curl command allocation failed");

    const pipe = c.popen(@ptrCast(cmd.items.ptr), "r") orelse return writeOut(out, out_len, "curl failed to start");
    var response: [4096]u8 = undefined;
    const line = c.fgets(&response, response.len, pipe);
    const status = c.pclose(pipe);
    if (status != 0) {
        return writeOut(out, out_len, "Last.fm request failed");
    }
    if (line == null) return writeOut(out, out_len, "Last.fm accepted the request");
    var len: usize = 0;
    while (len < response.len and response[len] != 0 and response[len] != '\n' and response[len] != '\r') {
        len += 1;
    }
    if (len == 0) return writeOut(out, out_len, "Last.fm accepted the request");
    return writeOut(out, out_len, response[0..len]);
}

pub fn main() !void {
    const callbacks = CrepusScrobblerCallbacks{
        .user_data = null,
        .current_track = currentTrack,
        .now_playing = nowPlaying,
        .scrobble = scrobble,
    };
    const code = crepus_local_scrobbler_run(callbacks);
    if (code != 0) return error.CrepusRunFailed;
}
