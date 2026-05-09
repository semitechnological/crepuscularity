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

data class ViewStyle(
    val padding: Float? = null,
    val paddingHorizontal: Float? = null,
    val paddingVertical: Float? = null,
    val paddingTop: Float? = null,
    val paddingBottom: Float? = null,
    val paddingLeft: Float? = null,
    val paddingRight: Float? = null,
    val margin: Float? = null,
    val marginHorizontal: Float? = null,
    val marginVertical: Float? = null,
    val marginTop: Float? = null,
    val marginBottom: Float? = null,
    val marginLeft: Float? = null,
    val marginRight: Float? = null,
    val fontSize: Float? = null,
    val fontWeight: Int? = null,
    val textAlign: String? = null,
    val foregroundColor: String? = null,
    val backgroundColor: String? = null,
    val cornerRadius: Float? = null,
    val italic: Boolean? = null,
    val underline: Boolean? = null,
    val strikethrough: Boolean? = null,
)

sealed interface ViewNode {
    data class Text(
        val content: String,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Stack(
        val axis: String,
        val spacing: Float? = null,
        val alignItems: String? = null,
        val justifyContent: String? = null,
        val style: ViewStyle? = null,
        val children: List<ViewNode> = emptyList(),
    ) : ViewNode

    data class Button(
        val label: String,
        val onClick: String? = null,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Image(
        val src: String,
        val alt: String? = null,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Scroll(
        val axis: String,
        val style: ViewStyle? = null,
        val children: List<ViewNode> = emptyList(),
    ) : ViewNode

    data class SlotRotate(
        val phrases: List<String>,
        val intervalMs: Long,
        val style: ViewStyle? = null,
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

private fun decodeStyle(o: JsonObject?): ViewStyle? {
    if (o == null) return null
    fun f(key: String): Float? = o[key]?.jsonPrimitive?.content?.toFloatOrNull()
    fun i(key: String): Int? = o[key]?.jsonPrimitive?.content?.toIntOrNull()
    fun b(key: String): Boolean? = o[key]?.jsonPrimitive?.content?.toBooleanStrictOrNull()
    fun s(key: String): String? = o[key]?.jsonPrimitive?.content
    return ViewStyle(
        padding = f("padding"),
        paddingHorizontal = f("paddingHorizontal"),
        paddingVertical = f("paddingVertical"),
        paddingTop = f("paddingTop"),
        paddingBottom = f("paddingBottom"),
        paddingLeft = f("paddingLeft"),
        paddingRight = f("paddingRight"),
        margin = f("margin"),
        marginHorizontal = f("marginHorizontal"),
        marginVertical = f("marginVertical"),
        marginTop = f("marginTop"),
        marginBottom = f("marginBottom"),
        marginLeft = f("marginLeft"),
        marginRight = f("marginRight"),
        fontSize = f("fontSize"),
        fontWeight = i("fontWeight"),
        textAlign = s("textAlign"),
        foregroundColor = s("foregroundColor"),
        backgroundColor = s("backgroundColor"),
        cornerRadius = f("cornerRadius"),
        italic = b("italic"),
        underline = b("underline"),
        strikethrough = b("strikethrough"),
    ).takeUnless { it.isEmpty() }
}

private fun ViewStyle.isEmpty(): Boolean =
    padding == null && paddingHorizontal == null && paddingVertical == null &&
        paddingTop == null && paddingBottom == null && paddingLeft == null && paddingRight == null &&
        margin == null && marginHorizontal == null && marginVertical == null &&
        marginTop == null && marginBottom == null && marginLeft == null && marginRight == null &&
        fontSize == null && fontWeight == null && textAlign == null &&
        foregroundColor == null && backgroundColor == null && cornerRadius == null &&
        italic == null && underline == null && strikethrough == null

private fun decodeNode(o: JsonObject): ViewNode {
    val kind = o["kind"]!!.jsonPrimitive.content
    return when (kind) {
        "text" -> ViewNode.Text(
            content = o["content"]!!.jsonPrimitive.content,
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "stack" -> ViewNode.Stack(
            axis = o["axis"]!!.jsonPrimitive.content,
            spacing = o["spacing"]?.jsonPrimitive?.content?.toFloatOrNull(),
            alignItems = o["alignItems"]?.jsonPrimitive?.content,
            justifyContent = o["justifyContent"]?.jsonPrimitive?.content,
            style = decodeStyle(o["style"]?.jsonObject),
            children = o["children"]?.jsonArray?.map { decodeNode(it.jsonObject) } ?: emptyList(),
        )
        "button" -> ViewNode.Button(
            label = o["label"]!!.jsonPrimitive.content,
            onClick = o["onClick"]?.jsonPrimitive?.content,
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "image" -> ViewNode.Image(
            src = o["src"]!!.jsonPrimitive.content,
            alt = o["alt"]?.jsonPrimitive?.content,
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "scroll" -> ViewNode.Scroll(
            axis = o["axis"]!!.jsonPrimitive.content,
            style = decodeStyle(o["style"]?.jsonObject),
            children = o["children"]?.jsonArray?.map { decodeNode(it.jsonObject) } ?: emptyList(),
        )
        "slotRotate" -> ViewNode.SlotRotate(
            phrases = o["phrases"]!!.jsonArray.map { it.jsonPrimitive.content },
            intervalMs = o["intervalMs"]!!.jsonPrimitive.content.toLong(),
            style = decodeStyle(o["style"]?.jsonObject),
        )
        else -> error("unknown ViewNode kind: $kind")
    }
}
