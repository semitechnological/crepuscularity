using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Crepuscularity.Plugin;

public sealed record ViewIr(
    [property: JsonPropertyName("version")] int Version,
    [property: JsonPropertyName("root")] JsonElement Root
);

public static class Crepuscularity
{
    public static async Task<ViewIr> RenderIrAsync(string path, object? context = null)
    {
        var template = await File.ReadAllTextAsync(path);
        var payload = JsonSerializer.Serialize(new { template, context = context ?? new { } });
        var bin = Environment.GetEnvironmentVariable("CREPUS_BIN") ?? "crepus";
        using var proc = new Process();
        proc.StartInfo = new ProcessStartInfo(bin, "native ir --stdin-json")
        {
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true
        };
        proc.Start();
        await proc.StandardInput.WriteAsync(payload);
        proc.StandardInput.Close();
        var stdout = await proc.StandardOutput.ReadToEndAsync();
        var stderr = await proc.StandardError.ReadToEndAsync();
        await proc.WaitForExitAsync();
        if (proc.ExitCode != 0)
        {
            throw new InvalidOperationException(stderr);
        }
        return JsonSerializer.Deserialize<ViewIr>(stdout) ?? throw new InvalidOperationException("empty IR");
    }
}
