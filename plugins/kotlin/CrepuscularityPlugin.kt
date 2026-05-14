package dev.crepuscularity.plugin

import java.io.ByteArrayOutputStream

data class ViewIr(val version: Int, val json: String)

object CrepuscularityPlugin {
    fun renderIr(path: String): ViewIr {
        val bin = System.getenv("CREPUS_BIN") ?: "crepus"
        val process = ProcessBuilder(bin, "native", "ir", path).start()
        val stdout = ByteArrayOutputStream()
        process.inputStream.transferTo(stdout)
        val code = process.waitFor()
        val json = stdout.toString(Charsets.UTF_8)
        if (code != 0) error("crepus native ir failed")
        val version = if (json.contains("\"version\":3") || json.contains("\"version\": 3")) 3 else -1
        return ViewIr(version, json)
    }

    fun renderHtml(path: String): String {
        val content = renderIr(path).json.substringAfter("\"content\":\"", "").substringBefore("\"")
        return "<div data-crepus-kind=\"stack\" data-axis=\"column\">${escapeHtml(content)}</div>"
    }

    private fun escapeHtml(value: String): String =
        value.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\"", "&quot;")
}
