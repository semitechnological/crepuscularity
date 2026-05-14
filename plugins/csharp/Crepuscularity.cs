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

    public static async Task<string> RenderHtmlAsync(string path, object? context = null)
    {
        var ir = await RenderIrAsync(path, context);
        return string.Concat(ir.Root.EnumerateArray().Select(RenderNode));
    }

    private static string RenderNode(JsonElement node)
    {
        var kind = node.TryGetProperty("kind", out var kindElement) ? kindElement.GetString() : "";
        return kind switch
        {
            "text" => System.Net.WebUtility.HtmlEncode(node.GetProperty("content").GetString() ?? ""),
            "stack" or "scroll" => $"<div data-crepus-kind=\"{System.Net.WebUtility.HtmlEncode(kind)}\" data-axis=\"{System.Net.WebUtility.HtmlEncode(node.GetProperty("axis").GetString() ?? "column")}\">{string.Concat(node.GetProperty("children").EnumerateArray().Select(RenderNode))}</div>",
            "button" => $"<button>{System.Net.WebUtility.HtmlEncode(node.GetProperty("label").GetString() ?? "")}</button>",
            _ => ""
        };
    }
}
