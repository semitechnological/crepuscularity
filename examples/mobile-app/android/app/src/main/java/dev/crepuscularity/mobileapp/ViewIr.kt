package dev.crepuscularity.mobileapp

import org.json.JSONArray
import org.json.JSONObject

// ── IR Model ──────────────────────────────────────────────────────────────────

data class ViewIr(val version: Int, val root: List<ViewNode>)

sealed interface ViewNode {
    data class Text(val content: String?, val bind: String?, val style: ViewStyle?) : ViewNode
    data class Button(val label: String, val onClick: String?, val style: ViewStyle?) : ViewNode
    data class Toggle(val label: String?, val bind: String?, val checked: Boolean, val onChange: String?) : ViewNode
    data class Slider(val bind: String?, val value: Float, val min: Float, val max: Float, val step: Float) : ViewNode
    data class Input(val placeholder: String?, val bind: String?, val multiline: Boolean) : ViewNode
    data class Stack(val axis: String, val spacing: Float, val children: List<ViewNode>, val style: ViewStyle?) : ViewNode
    data class Scroll(val axis: String, val children: List<ViewNode>, val style: ViewStyle?) : ViewNode
    data class IfNode(val condition: String, val thenChildren: List<ViewNode>, val elseChildren: List<ViewNode>) : ViewNode
    data class ForEach(val bind: String, val itemName: String?, val itemBody: List<ViewNode>) : ViewNode
    data class Tabs(val bind: String?, val children: List<ViewNode>, val style: ViewStyle?) : ViewNode
    data class Tab(val value: String, val label: String, val icon: String?, val children: List<ViewNode>) : ViewNode
    data object Unknown : ViewNode
}

data class ViewStyle(
    val padding: Float? = null,
    val paddingHorizontal: Float? = null,
    val paddingVertical: Float? = null,
    val fontSize: Float? = null,
    val fontWeight: Int? = null,
    val foregroundColor: String? = null,
    val backgroundColor: String? = null,
    val cornerRadius: Float? = null,
    val flexGrow: Float? = null,
)

// ── Decoder ───────────────────────────────────────────────────────────────────

fun decodeViewIr(json: String): ViewIr {
    val obj = JSONObject(json)
    val version = obj.getInt("version")
    val root = decodeNodeArray(obj.getJSONArray("root"))
    return ViewIr(version, root)
}

private fun decodeNodeArray(arr: JSONArray): List<ViewNode> =
    (0 until arr.length()).map { decodeNode(arr.getJSONObject(it)) }

private fun decodeNode(obj: JSONObject): ViewNode {
    return when (obj.getString("kind")) {
        "text" -> ViewNode.Text(
            content = obj.optString("content", "").ifEmpty { null },
            bind = obj.optString("bind", "").ifEmpty { null },
            style = obj.optJSONObject("style")?.let { decodeStyle(it) }
        )
        "button" -> ViewNode.Button(
            label = obj.getString("label"),
            onClick = obj.optString("onClick", "").ifEmpty { null },
            style = obj.optJSONObject("style")?.let { decodeStyle(it) }
        )
        "toggle" -> ViewNode.Toggle(
            label = obj.optString("label", "").ifEmpty { null },
            bind = obj.optString("bind", "").ifEmpty { null },
            checked = obj.optBoolean("checked", false),
            onChange = obj.optString("onChange", "").ifEmpty { null }
        )
        "slider" -> ViewNode.Slider(
            bind = obj.optString("bind", "").ifEmpty { null },
            value = obj.optDouble("value", 0.0).toFloat(),
            min = obj.optDouble("min", 0.0).toFloat(),
            max = obj.optDouble("max", 100.0).toFloat(),
            step = obj.optDouble("step", 1.0).toFloat()
        )
        "input" -> ViewNode.Input(
            placeholder = obj.optString("placeholder", "").ifEmpty { null },
            bind = obj.optString("bind", "").ifEmpty { null },
            multiline = obj.optBoolean("multiline", false)
        )
        "stack" -> ViewNode.Stack(
            axis = obj.optString("axis", "column"),
            spacing = obj.optDouble("spacing", 0.0).toFloat(),
            children = obj.optJSONArray("children")?.let { decodeNodeArray(it) } ?: emptyList(),
            style = obj.optJSONObject("style")?.let { decodeStyle(it) }
        )
        "scroll" -> ViewNode.Scroll(
            axis = obj.optString("axis", "column"),
            children = obj.optJSONArray("children")?.let { decodeNodeArray(it) } ?: emptyList(),
            style = obj.optJSONObject("style")?.let { decodeStyle(it) }
        )
        "if" -> ViewNode.IfNode(
            condition = obj.getString("condition"),
            thenChildren = obj.optJSONArray("thenChildren")?.let { decodeNodeArray(it) } ?: emptyList(),
            elseChildren = obj.optJSONArray("elseChildren")?.let { decodeNodeArray(it) } ?: emptyList()
        )
        "forEach" -> ViewNode.ForEach(
            bind = obj.getString("bind"),
            itemName = obj.optString("itemName", "").ifEmpty { null },
            itemBody = obj.optJSONArray("itemBody")?.let { decodeNodeArray(it) } ?: emptyList()
        )
        "tabs" -> ViewNode.Tabs(
            bind = obj.optString("bind", "").ifEmpty { null },
            children = obj.optJSONArray("children")?.let { decodeNodeArray(it) } ?: emptyList(),
            style = obj.optJSONObject("style")?.let { decodeStyle(it) }
        )
        "tab" -> ViewNode.Tab(
            value = obj.getString("value"),
            label = obj.getString("label"),
            icon = obj.optString("icon", "").ifEmpty { null },
            children = obj.optJSONArray("children")?.let { decodeNodeArray(it) } ?: emptyList()
        )
        else -> ViewNode.Unknown
    }
}

private fun decodeStyle(obj: JSONObject): ViewStyle = ViewStyle(
    padding = obj.optFloatOrNull("padding"),
    paddingHorizontal = obj.optFloatOrNull("paddingHorizontal"),
    paddingVertical = obj.optFloatOrNull("paddingVertical"),
    fontSize = obj.optFloatOrNull("fontSize"),
    fontWeight = obj.optIntOrNull("fontWeight"),
    foregroundColor = obj.optString("foregroundColor", "").ifEmpty { null },
    backgroundColor = obj.optString("backgroundColor", "").ifEmpty { null },
    cornerRadius = obj.optFloatOrNull("cornerRadius"),
    flexGrow = obj.optFloatOrNull("flexGrow"),
)

private fun JSONObject.optFloatOrNull(key: String): Float? =
    if (has(key) && !isNull(key)) optDouble(key, 0.0).toFloat() else null

private fun JSONObject.optIntOrNull(key: String): Int? =
    if (has(key) && !isNull(key)) optInt(key, 0) else null
