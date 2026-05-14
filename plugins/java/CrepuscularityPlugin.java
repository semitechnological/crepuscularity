package dev.crepuscularity.plugin;

import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.util.List;

public final class CrepuscularityPlugin {
    public record ViewIr(int version, String json) {}

    public static ViewIr renderIr(String path) throws Exception {
        String bin = System.getenv().getOrDefault("CREPUS_BIN", "crepus");
        Process process = new ProcessBuilder(List.of(bin, "native", "ir", path)).start();
        ByteArrayOutputStream stdout = new ByteArrayOutputStream();
        process.getInputStream().transferTo(stdout);
        int code = process.waitFor();
        String json = stdout.toString(StandardCharsets.UTF_8);
        if (code != 0) {
            throw new IllegalStateException("crepus native ir failed");
        }
        int version = json.contains("\"version\":2") || json.contains("\"version\": 2") ? 2 : -1;
        return new ViewIr(version, json);
    }
}
