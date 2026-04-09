package dev.crepuscularity.nativeshell

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

data class ViewIr(
    val version: Int,
    val root: List<ViewNode>,
)

sealed interface ViewNode {
    data class Text(
        val content: String,
    ) : ViewNode

    data class Stack(
        val axis: String,
        val spacing: Float?,
        val children: List<ViewNode>,
    ) : ViewNode
}

private val jsonLenient = Json {
    ignoreUnknownKeys = true
    isLenient = true
}

fun decodeViewIr(jsonText: String): ViewIr {
    val doc = jsonLenient.parseToJsonElement(jsonText).jsonObject
    val version = doc["version"]!!.jsonPrimitive.content.toInt()
    val nodes = doc["root"]!!.jsonArray.map { decodeNode(it.jsonObject) }
    return ViewIr(version, nodes)
}

private fun decodeNode(o: JsonObject): ViewNode {
    val kind = o["kind"]!!.jsonPrimitive.content
    return when (kind) {
        "text" -> ViewNode.Text(content = o["content"]!!.jsonPrimitive.content)
        "stack" -> {
            val spacingEl = o["spacing"]?.jsonPrimitive
            val spacing =
                spacingEl?.content?.toFloatOrNull()
            ViewNode.Stack(
                axis = o["axis"]!!.jsonPrimitive.content,
                spacing = spacing,
                children = o["children"]?.jsonArray?.map { decodeNode(it.jsonObject) } ?: emptyList(),
            )
        }
        else -> error("unknown ViewNode kind: $kind")
    }
}
