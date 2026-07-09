package dev.crepuscularity.mobileapp

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun ViewIrRoot(ir: ViewIr) {
    Column {
        ir.root.forEach { node -> ViewNodeComposable(node) }
    }
}

@Composable
fun ViewNodeComposable(node: ViewNode) {
    when (node) {
        is ViewNode.Text -> TextNode(node)
        is ViewNode.Button -> ButtonNode(node)
        is ViewNode.Toggle -> ToggleNode(node)
        is ViewNode.Slider -> SliderNode(node)
        is ViewNode.Input -> InputNode(node)
        is ViewNode.Stack -> StackNode(node)
        is ViewNode.Scroll -> ScrollNode(node)
        is ViewNode.IfNode -> IfNode(node)
        is ViewNode.ForEach -> ForEachNode(node)
        is ViewNode.Tabs -> TabsNode(node)
        is ViewNode.Tab -> TabNode(node)
        is ViewNode.Unknown -> {}
    }
}

@Composable
private fun TextNode(node: ViewNode.Text) {
    val text = node.content ?: node.bind ?: ""
    val style = node.style
    Text(
        text = text,
        fontSize = style?.fontSize?.sp ?: 14.sp,
        fontWeight = style?.fontWeight?.let { FontWeight(it) },
        color = parseColor(style?.foregroundColor),
    )
}

@Composable
private fun ButtonNode(node: ViewNode.Button) {
    val style = node.style
    Button(
        onClick = { /* ponytail: action dispatch placeholder */ },
        colors = ButtonDefaults.buttonColors(
            containerColor = parseColor(style?.backgroundColor) ?: MaterialTheme.colorScheme.primary,
            contentColor = parseColor(style?.foregroundColor) ?: MaterialTheme.colorScheme.onPrimary,
        ),
        shape = RoundedCornerShape((style?.cornerRadius ?: 8f).dp),
        contentPadding = PaddingValues(
            horizontal = (style?.paddingHorizontal ?: style?.padding ?: 16f).dp,
            vertical = (style?.paddingVertical ?: style?.padding ?: 8f).dp,
        ),
    ) {
        Text(node.label, fontSize = style?.fontSize?.sp ?: 14.sp, fontWeight = style?.fontWeight?.let { FontWeight(it) })
    }
}

@Composable
private fun ToggleNode(node: ViewNode.Toggle) {
    var checked by remember { mutableStateOf(node.checked) }
    Row(verticalAlignment = Alignment.CenterVertically) {
        if (!node.label.isNullOrEmpty()) {
            Text(node.label, modifier = Modifier.weight(1f))
        }
        Switch(checked = checked, onCheckedChange = { checked = it })
    }
}

@Composable
private fun SliderNode(node: ViewNode.Slider) {
    var value by remember { mutableFloatStateOf(node.value) }
    Slider(value = value, onValueChange = { value = it }, valueRange = node.min..node.max, steps = ((node.max - node.min) / node.step - 1).toInt().coerceAtLeast(0))
}

@Composable
private fun InputNode(node: ViewNode.Input) {
    var text by remember { mutableStateOf("") }
    OutlinedTextField(
        value = text,
        onValueChange = { text = it },
        placeholder = { node.placeholder?.let { Text(it) } },
        modifier = Modifier.fillMaxWidth(),
    )
}

@Composable
private fun StackNode(node: ViewNode.Stack) {
    val spacing = node.spacing.dp
    val style = node.style
    val mod = Modifier
        .then(style?.backgroundColor?.let { Modifier.background(parseColor(it) ?: Color.Transparent) } ?: Modifier)
        .then(style?.paddingHorizontal?.let { Modifier.padding(horizontal = it.dp) } ?: Modifier)
        .then(style?.paddingVertical?.let { Modifier.padding(vertical = it.dp) } ?: Modifier)
        .then(style?.cornerRadius?.let { Modifier.clip(RoundedCornerShape(it.dp)) } ?: Modifier)

    when (node.axis) {
        "row" -> Row(horizontalArrangement = Arrangement.spacedBy(spacing), modifier = mod) {
            node.children.forEach { ViewNodeComposable(it) }
        }
        else -> Column(verticalArrangement = Arrangement.spacedBy(spacing), modifier = mod) {
            node.children.forEach { ViewNodeComposable(it) }
        }
    }
}

@Composable
private fun ScrollNode(node: ViewNode.Scroll) {
    val modifier = when (node.axis) {
        "row" -> Modifier.horizontalScroll(rememberScrollState())
        else -> Modifier.verticalScroll(rememberScrollState())
    }
    Column(modifier = modifier) {
        node.children.forEach { ViewNodeComposable(it) }
    }
}

@Composable
private fun IfNode(node: ViewNode.IfNode) {
    // ponytail: static render shows thenChildren; real apps evaluate condition
    node.thenChildren.forEach { ViewNodeComposable(it) }
}

@Composable
private fun ForEachNode(node: ViewNode.ForEach) {
    // ponytail: static render shows itemBody once; real apps iterate bound data
    node.itemBody.forEach { ViewNodeComposable(it) }
}

@Composable
private fun TabsNode(node: ViewNode.Tabs) {
    val tabs = node.children.filterIsInstance<ViewNode.Tab>()
    var selected by remember { mutableIntStateOf(0) }
    Column {
        TabRow(selectedTabIndex = selected) {
            tabs.forEachIndexed { index, tab ->
                Tab(selected = selected == index, onClick = { selected = index }, text = { Text(tab.label) })
            }
        }
        tabs.getOrNull(selected)?.let { tab ->
            tab.children.forEach { ViewNodeComposable(it) }
        }
    }
}

@Composable
private fun TabNode(node: ViewNode.Tab) {
    node.children.forEach { ViewNodeComposable(it) }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

private fun parseColor(hex: String?): Color? {
    if (hex.isNullOrEmpty()) return null
    val cleaned = if (hex.startsWith("#")) hex.drop(1) else hex
    return try {
        val value = cleaned.toLong(16)
        Color(0xFF000000 or value)
    } catch (_: Exception) {
        null
    }
}

private fun Modifier.clip(shape: RoundedCornerShape): Modifier = this.then(Modifier.background(Color.Transparent, shape))
