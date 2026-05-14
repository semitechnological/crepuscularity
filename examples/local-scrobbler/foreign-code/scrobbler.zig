const c = @cImport({
    @cInclude("stdio.h");
});

const spotify_cmd =
    "osascript -e 'tell application \"Spotify\" to if it is running then artist of current track & \"\t\" & name of current track & \"\t\" & album of current track & \"\t\" & player state as string else \"stopped\"' 2>/dev/null";
const music_cmd =
    "osascript -e 'tell application \"Music\" to if it is running then artist of current track & \"\t\" & name of current track & \"\t\" & album of current track & \"\t\" & player state as string else \"stopped\"' 2>/dev/null";

fn writeOut(out: [*]u8, out_len: usize, value: []const u8) callconv(.c) c_int {
    if (out_len == 0) return -1;
    const n = @min(value.len, out_len - 1);
    @memcpy(out[0..n], value[0..n]);
    out[n] = 0;
    return @intCast(n);
}

export fn crepus_current_track(source: c_int, out: [*]u8, out_len: usize) c_int {
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
