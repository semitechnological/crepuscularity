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
    val width: Float? = null,
    val height: Float? = null,
    val minWidth: Float? = null,
    val minHeight: Float? = null,
    val maxWidth: Float? = null,
    val maxHeight: Float? = null,
    val fontSize: Float? = null,
    val fontWeight: Int? = null,
    val textAlign: String? = null,
    val foregroundColor: String? = null,
    val backgroundColor: String? = null,
    val cornerRadius: Float? = null,
    val borderWidth: Float? = null,
    val borderColor: String? = null,
    val italic: Boolean? = null,
    val underline: Boolean? = null,
    val strikethrough: Boolean? = null,
)

data class PickerOption(
    val value: String,
    val label: String,
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

    data class Toggle(
        val label: String,
        val bind: String? = null,
        val checked: Boolean,
        val onChange: String? = null,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Checkbox(
        val label: String,
        val bind: String? = null,
        val checked: Boolean,
        val onChange: String? = null,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Slider(
        val label: String? = null,
        val bind: String? = null,
        val value: Float,
        val min: Float,
        val max: Float,
        val step: Float? = null,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Progress(
        val label: String? = null,
        val value: Float,
        val max: Float,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Meter(
        val label: String? = null,
        val value: Float,
        val min: Float,
        val max: Float,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Badge(
        val label: String,
        val tone: String? = null,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Divider(
        val axis: String,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Spacer(
        val size: Float? = null,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Dropzone(
        val label: String,
        val accept: String? = null,
        val onDrop: String? = null,
        val style: ViewStyle? = null,
        val children: List<ViewNode> = emptyList(),
    ) : ViewNode

    data class FilePicker(
        val label: String,
        val accept: List<String> = emptyList(),
        val multiple: Boolean = false,
        val onPick: String? = null,
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

    data class ListNode(
        val ordered: Boolean,
        val style: ViewStyle? = null,
        val children: List<ViewNode> = emptyList(),
    ) : ViewNode

    data class ListItem(
        val style: ViewStyle? = null,
        val children: List<ViewNode> = emptyList(),
    ) : ViewNode

    data class SlotRotate(
        val phrases: List<String>,
        val intervalMs: Long,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Input(
        val placeholder: String,
        val bind: String,
        val multiline: Boolean = false,
        val style: ViewStyle? = null,
    ) : ViewNode

    data class Picker(
        val bind: String,
        val options: List<PickerOption>,
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
        width = f("width"),
        height = f("height"),
        minWidth = f("minWidth"),
        minHeight = f("minHeight"),
        maxWidth = f("maxWidth"),
        maxHeight = f("maxHeight"),
        fontSize = f("fontSize"),
        fontWeight = i("fontWeight"),
        textAlign = s("textAlign"),
        foregroundColor = s("foregroundColor"),
        backgroundColor = s("backgroundColor"),
        cornerRadius = f("cornerRadius"),
        borderWidth = f("borderWidth"),
        borderColor = s("borderColor"),
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
        width == null && height == null && minWidth == null && minHeight == null &&
        maxWidth == null && maxHeight == null &&
        fontSize == null && fontWeight == null && textAlign == null &&
        foregroundColor == null && backgroundColor == null && cornerRadius == null &&
        borderWidth == null && borderColor == null &&
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
        "toggle" -> ViewNode.Toggle(
            label = o["label"]!!.jsonPrimitive.content,
            bind = o["bind"]?.jsonPrimitive?.content,
            checked = o["checked"]!!.jsonPrimitive.content.toBoolean(),
            onChange = o["onChange"]?.jsonPrimitive?.content,
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "checkbox" -> ViewNode.Checkbox(
            label = o["label"]!!.jsonPrimitive.content,
            bind = o["bind"]?.jsonPrimitive?.content,
            checked = o["checked"]!!.jsonPrimitive.content.toBoolean(),
            onChange = o["onChange"]?.jsonPrimitive?.content,
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "slider" -> ViewNode.Slider(
            label = o["label"]?.jsonPrimitive?.content,
            bind = o["bind"]?.jsonPrimitive?.content,
            value = o["value"]!!.jsonPrimitive.content.toFloat(),
            min = o["min"]!!.jsonPrimitive.content.toFloat(),
            max = o["max"]!!.jsonPrimitive.content.toFloat(),
            step = o["step"]?.jsonPrimitive?.content?.toFloatOrNull(),
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "progress" -> ViewNode.Progress(
            label = o["label"]?.jsonPrimitive?.content,
            value = o["value"]!!.jsonPrimitive.content.toFloat(),
            max = o["max"]!!.jsonPrimitive.content.toFloat(),
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "meter" -> ViewNode.Meter(
            label = o["label"]?.jsonPrimitive?.content,
            value = o["value"]!!.jsonPrimitive.content.toFloat(),
            min = o["min"]!!.jsonPrimitive.content.toFloat(),
            max = o["max"]!!.jsonPrimitive.content.toFloat(),
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "badge" -> ViewNode.Badge(
            label = o["label"]!!.jsonPrimitive.content,
            tone = o["tone"]?.jsonPrimitive?.content,
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "divider" -> ViewNode.Divider(
            axis = o["axis"]!!.jsonPrimitive.content,
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "spacer" -> ViewNode.Spacer(
            size = o["size"]?.jsonPrimitive?.content?.toFloatOrNull(),
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "dropzone" -> ViewNode.Dropzone(
            label = o["label"]!!.jsonPrimitive.content,
            accept = o["accept"]?.jsonPrimitive?.content,
            onDrop = o["onDrop"]?.jsonPrimitive?.content,
            style = decodeStyle(o["style"]?.jsonObject),
            children = o["children"]?.jsonArray?.map { decodeNode(it.jsonObject) } ?: emptyList(),
        )
        "filePicker" -> ViewNode.FilePicker(
            label = o["label"]!!.jsonPrimitive.content,
            accept = o["accept"]?.jsonArray?.map { it.jsonPrimitive.content } ?: emptyList(),
            multiple = o["multiple"]?.jsonPrimitive?.content?.toBoolean() ?: false,
            onPick = o["onPick"]?.jsonPrimitive?.content,
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
        "list" -> ViewNode.ListNode(
            ordered = o["ordered"]!!.jsonPrimitive.content.toBoolean(),
            style = decodeStyle(o["style"]?.jsonObject),
            children = o["children"]?.jsonArray?.map { decodeNode(it.jsonObject) } ?: emptyList(),
        )
        "listItem" -> ViewNode.ListItem(
            style = decodeStyle(o["style"]?.jsonObject),
            children = o["children"]?.jsonArray?.map { decodeNode(it.jsonObject) } ?: emptyList(),
        )
        "slotRotate" -> ViewNode.SlotRotate(
            phrases = o["phrases"]!!.jsonArray.map { it.jsonPrimitive.content },
            intervalMs = o["intervalMs"]!!.jsonPrimitive.content.toLong(),
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "input" -> ViewNode.Input(
            placeholder = o["placeholder"]!!.jsonPrimitive.content,
            bind = o["bind"]!!.jsonPrimitive.content,
            multiline = o["multiline"]?.jsonPrimitive?.content?.toBoolean() ?: false,
            style = decodeStyle(o["style"]?.jsonObject),
        )
        "picker" -> ViewNode.Picker(
            bind = o["bind"]!!.jsonPrimitive.content,
            options = o["options"]!!.jsonArray.map {
                val option = it.jsonObject
                PickerOption(
                    value = option["value"]!!.jsonPrimitive.content,
                    label = option["label"]!!.jsonPrimitive.content,
                )
            },
            style = decodeStyle(o["style"]?.jsonObject),
        )
        else -> error("unknown ViewNode kind: $kind")
    }
}
