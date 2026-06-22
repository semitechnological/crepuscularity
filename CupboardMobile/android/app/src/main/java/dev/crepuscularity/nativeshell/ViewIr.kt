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

data class HotReloadEnvelope(
    val sequence: Long,
    val message: HotReloadMessage,
)

sealed interface HotReloadMessage {
    data object Noop : HotReloadMessage
    data class Patch(
        val mutations: List<IrMutation>,
    ) : HotReloadMessage
    data class FullReload(
        val ir: ViewIr,
        val reason: String,
    ) : HotReloadMessage
    data class Error(
        val message: String,
    ) : HotReloadMessage
    data object Unsupported : HotReloadMessage
}

sealed interface IrMutation {
    data class ReplaceRoot(
        val root: List<ViewNode>,
    ) : IrMutation
    data class ReplaceNode(
        val path: List<Int>,
        val node: ViewNode,
    ) : IrMutation
    data class InsertNode(
        val parentPath: List<Int>,
        val index: Int,
        val node: ViewNode,
    ) : IrMutation
    data class RemoveNode(
        val path: List<Int>,
    ) : IrMutation
    data class UpdateText(
        val path: List<Int>,
        val content: String,
    ) : IrMutation
    data class UpdateStyle(
        val path: List<Int>,
        val style: ViewStyle?,
    ) : IrMutation
}

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

fun decodeHotReloadEnvelope(jsonText: String): HotReloadEnvelope {
    val doc = jsonLenient.parseToJsonElement(jsonText).jsonObject
    val sequence = doc["sequence"]?.jsonPrimitive?.content?.toLongOrNull() ?: 0L
    val message = decodeHotReloadMessage(doc["message"]?.jsonObject ?: error("missing message"))
    return HotReloadEnvelope(sequence, message)
}

fun ViewIr.apply(envelope: HotReloadEnvelope): ViewIr =
    when (val message = envelope.message) {
        HotReloadMessage.Noop, HotReloadMessage.Unsupported -> this
        is HotReloadMessage.Error -> error(message.message)
        is HotReloadMessage.FullReload -> message.ir
        is HotReloadMessage.Patch -> apply(message.mutations)
    }

fun ViewIr.apply(mutations: List<IrMutation>): ViewIr {
    var current = this
    for (mutation in mutations) {
        current = current.apply(mutation)
    }
    return current
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
        italic = b("italic"),
        underline = b("underline"),
        strikethrough = b("strikethrough"),
    ).takeUnless { it.isEmpty() }
}

private fun decodeHotReloadMessage(o: JsonObject): HotReloadMessage =
    when (o["kind"]?.jsonPrimitive?.content) {
        "noop" -> HotReloadMessage.Noop
        "patch" -> HotReloadMessage.Patch(
            mutations = o["mutations"]?.jsonArray?.map { decodeMutation(it.jsonObject) } ?: emptyList(),
        )
        "fullReload" -> HotReloadMessage.FullReload(
            ir = decodeViewIr(Json.encodeToString(JsonObject.serializer(), o["ir"]!!.jsonObject)),
            reason = o["reason"]?.jsonPrimitive?.content ?: "",
        )
        "error" -> HotReloadMessage.Error(
            message = o["message"]?.jsonPrimitive?.content ?: "unknown hot reload error",
        )
        else -> HotReloadMessage.Unsupported
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

private fun decodeMutation(o: JsonObject): IrMutation =
    when (o["op"]?.jsonPrimitive?.content) {
        "replaceRoot" -> IrMutation.ReplaceRoot(
            root = o["root"]?.jsonArray?.map { decodeNode(it.jsonObject) } ?: emptyList(),
        )
        "replaceNode" -> IrMutation.ReplaceNode(
            path = decodePath(o["path"]?.jsonArray),
            node = decodeNode(o["node"]!!.jsonObject),
        )
        "insertNode" -> IrMutation.InsertNode(
            parentPath = decodePath(o["parentPath"]?.jsonArray),
            index = o["index"]!!.jsonPrimitive.content.toInt(),
            node = decodeNode(o["node"]!!.jsonObject),
        )
        "removeNode" -> IrMutation.RemoveNode(
            path = decodePath(o["path"]?.jsonArray),
        )
        "updateText" -> IrMutation.UpdateText(
            path = decodePath(o["path"]?.jsonArray),
            content = o["content"]!!.jsonPrimitive.content,
        )
        "updateStyle" -> IrMutation.UpdateStyle(
            path = decodePath(o["path"]?.jsonArray),
            style = decodeStyle(o["style"]?.jsonObject),
        )
        else -> error("unknown mutation op: ${o["op"]?.jsonPrimitive?.content}")
    }

private fun decodePath(path: kotlinx.serialization.json.JsonArray?): List<Int> =
    path?.map { it.jsonPrimitive.content.toInt() } ?: emptyList()

private fun ViewIr.apply(mutation: IrMutation): ViewIr =
    when (mutation) {
        is IrMutation.ReplaceRoot -> copy(root = mutation.root)
        is IrMutation.ReplaceNode -> copy(root = root.replaceNode(mutation.path, mutation.node))
        is IrMutation.InsertNode -> copy(root = root.insertNode(mutation.parentPath, mutation.index, mutation.node))
        is IrMutation.RemoveNode -> copy(root = root.removeNode(mutation.path))
        is IrMutation.UpdateText -> copy(root = root.updateText(mutation.path, mutation.content))
        is IrMutation.UpdateStyle -> copy(root = root.updateStyle(mutation.path, mutation.style))
    }

private fun List<ViewNode>.replaceNode(path: List<Int>, node: ViewNode): List<ViewNode> {
    require(path.isNotEmpty()) { "replaceNode requires a path" }
    val index = path.first()
    require(index in indices) { "invalid node path: $path" }
    return toMutableList().also { list ->
        list[index] =
            if (path.size == 1) node else list[index].replaceInChildren(path.drop(1), node)
    }
}

private fun List<ViewNode>.insertNode(parentPath: List<Int>, index: Int, node: ViewNode): List<ViewNode> {
    if (parentPath.isEmpty()) {
        require(index <= size) { "invalid insert index: $index" }
        return toMutableList().also { it.add(index, node) }
    }
    val parentIndex = parentPath.first()
    require(parentIndex in indices) { "invalid parent path: $parentPath" }
    return toMutableList().also { list ->
        list[parentIndex] = list[parentIndex].mutateChildren(parentPath.drop(1)) { children ->
            require(index <= children.size) { "invalid insert index: $index" }
            children.add(index, node)
        }
    }
}

private fun List<ViewNode>.removeNode(path: List<Int>): List<ViewNode> {
    require(path.isNotEmpty()) { "cannot remove root list directly" }
    if (path.size == 1) {
        val index = path.first()
        require(index in indices) { "invalid remove path: $path" }
        return toMutableList().also { it.removeAt(index) }
    }
    val parentIndex = path.first()
    require(parentIndex in indices) { "invalid remove path: $path" }
    return toMutableList().also { list ->
        list[parentIndex] = list[parentIndex].mutateChildren(path.drop(1).dropLast(1)) { children ->
            val index = path.last()
            require(index in children.indices) { "invalid remove path: $path" }
            children.removeAt(index)
        }
    }
}

private fun List<ViewNode>.updateText(path: List<Int>, content: String): List<ViewNode> {
    require(path.isNotEmpty()) { "updateText requires a path" }
    val index = path.first()
    require(index in indices) { "invalid text path: $path" }
    return toMutableList().also { list ->
        list[index] =
            if (path.size == 1) {
                when (val node = list[index]) {
                    is ViewNode.Text -> node.copy(content = content)
                    else -> error("UpdateText expects text node at $path")
                }
            } else {
                list[index].updateTextInChildren(path.drop(1), content)
            }
    }
}

private fun List<ViewNode>.updateStyle(path: List<Int>, style: ViewStyle?): List<ViewNode> {
    require(path.isNotEmpty()) { "updateStyle requires a path" }
    val index = path.first()
    require(index in indices) { "invalid style path: $path" }
    return toMutableList().also { list ->
        list[index] =
            if (path.size == 1) {
                list[index].withStyle(style)
            } else {
                list[index].updateStyleInChildren(path.drop(1), style)
            }
    }
}

private fun ViewNode.replaceInChildren(path: List<Int>, node: ViewNode): ViewNode {
    val index = path.first()
    return mutateChildren(emptyList()) { children ->
        require(index in children.indices) { "invalid node path: $path" }
        children[index] =
            if (path.size == 1) node else children[index].replaceInChildren(path.drop(1), node)
    }
}

private fun ViewNode.updateTextInChildren(path: List<Int>, content: String): ViewNode {
    val index = path.first()
    return mutateChildren(emptyList()) { children ->
        require(index in children.indices) { "invalid text path: $path" }
        children[index] =
            if (path.size == 1) {
                when (val child = children[index]) {
                    is ViewNode.Text -> child.copy(content = content)
                    else -> error("UpdateText expects text node at $path")
                }
            } else {
                children[index].updateTextInChildren(path.drop(1), content)
            }
    }
}

private fun ViewNode.updateStyleInChildren(path: List<Int>, style: ViewStyle?): ViewNode {
    val index = path.first()
    return mutateChildren(emptyList()) { children ->
        require(index in children.indices) { "invalid style path: $path" }
        children[index] =
            if (path.size == 1) {
                children[index].withStyle(style)
            } else {
                children[index].updateStyleInChildren(path.drop(1), style)
            }
    }
}

private fun ViewNode.mutateChildren(path: List<Int>, body: (MutableList<ViewNode>) -> Unit): ViewNode =
    when (this) {
        is ViewNode.Stack -> {
            val children = children.toMutableList()
            if (path.isEmpty()) {
                body(children)
            } else {
                val index = path.first()
                require(index in children.indices) { "invalid path: $path" }
                children[index] = children[index].mutateChildren(path.drop(1), body)
            }
            copy(children = children)
        }
        is ViewNode.Scroll -> {
            val children = children.toMutableList()
            if (path.isEmpty()) {
                body(children)
            } else {
                val index = path.first()
                require(index in children.indices) { "invalid path: $path" }
                children[index] = children[index].mutateChildren(path.drop(1), body)
            }
            copy(children = children)
        }
        else -> error("node does not have children")
    }

private fun ViewNode.withStyle(style: ViewStyle?): ViewNode =
    when (this) {
        is ViewNode.Text -> copy(style = style)
        is ViewNode.Stack -> copy(style = style)
        is ViewNode.Button -> copy(style = style)
        is ViewNode.Image -> copy(style = style)
        is ViewNode.Scroll -> copy(style = style)
        is ViewNode.SlotRotate -> copy(style = style)
    }
