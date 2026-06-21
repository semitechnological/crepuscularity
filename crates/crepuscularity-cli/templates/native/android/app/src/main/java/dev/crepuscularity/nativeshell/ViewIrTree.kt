package dev.crepuscularity.nativeshell

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.roundToInt

@Composable
fun ViewIrRoot(ir: ViewIr, modifier: Modifier = Modifier) {
    Column(modifier = modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(0.dp)) {
        ir.root.forEach { node -> ViewNodeView(node) }
    }
}

@Composable
fun ViewNodeView(node: ViewNode) {
    when (node) {
        is ViewNode.Text -> styledText(node.content, node.style)
        is ViewNode.Stack -> {
            val gap = (node.spacing ?: 8f).dp
            val arr = Arrangement.spacedBy(gap)
            val colAlign = columnAlign(node.alignItems)
            val rowAlign = rowAlign(node.alignItems)
            val mod = stackModifier(node.style)
            when (node.axis) {
                "row" ->
                    withContentColor(node.style) {
                        Row(
                            modifier = mod,
                            horizontalArrangement = arr,
                            verticalAlignment = rowAlign,
                        ) {
                            node.children.forEach { child -> ViewNodeView(child) }
                        }
                    }
                else ->
                    withContentColor(node.style) {
                        Column(
                            modifier = mod,
                            verticalArrangement = arr,
                            horizontalAlignment = colAlign,
                        ) {
                            node.children.forEach { child -> ViewNodeView(child) }
                        }
                    }
            }
        }
        is ViewNode.Button ->
            Button(onClick = { node.onClick?.let { CrepusActionState.dispatch(it) } }, modifier = stackModifier(node.style)) {
                Text(node.label)
            }
        is ViewNode.Toggle -> {
            var checked by remember { mutableStateOf(node.checked) }
            Row(
                modifier = stackModifier(node.style).fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(node.label)
                Switch(
                    checked = checked,
                    onCheckedChange = {
                        checked = it
                        node.onChange?.let(CrepusActionState::dispatch)
                    },
                )
            }
        }
        is ViewNode.Checkbox -> {
            var checked by remember { mutableStateOf(node.checked) }
            Row(
                modifier =
                    stackModifier(node.style)
                        .fillMaxWidth()
                        .clickable {
                            checked = !checked
                            node.onChange?.let(CrepusActionState::dispatch)
                        },
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Checkbox(
                    checked = checked,
                    onCheckedChange = {
                        checked = it
                        node.onChange?.let(CrepusActionState::dispatch)
                    },
                )
                Text(node.label)
            }
        }
        is ViewNode.Slider -> {
            var value by remember { mutableFloatStateOf(node.value) }
            Column(modifier = stackModifier(node.style), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                node.label?.let { Text("$it ${value.roundToInt()}") }
                androidx.compose.material3.Slider(
                    value = value,
                    onValueChange = { value = quantizeSlider(it, node.min, node.step) },
                    valueRange = node.min..node.max,
                    steps = sliderSteps(node.min, node.max, node.step),
                )
            }
        }
        is ViewNode.Progress ->
            Column(modifier = stackModifier(node.style), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                node.label?.let { Text(it) }
                LinearProgressIndicator(progress = { progressValue(node.value, 0f, node.max) }, modifier = Modifier.fillMaxWidth())
            }
        is ViewNode.Meter ->
            Column(modifier = stackModifier(node.style), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                node.label?.let { Text(it) }
                LinearProgressIndicator(progress = { progressValue(node.value, node.min, node.max) }, modifier = Modifier.fillMaxWidth())
            }
        is ViewNode.Badge ->
            Text(
                text = node.label,
                modifier =
                    stackModifier(node.style)
                        .clip(RoundedCornerShape(999.dp))
                        .background(badgeColor(node.tone))
                        .padding(horizontal = 10.dp, vertical = 6.dp),
                fontSize = 12.sp,
                color = Color.White,
            )
        is ViewNode.Divider ->
            when (node.axis) {
                "row" ->
                    Box(
                        modifier = stackModifier(node.style).width(1.dp).height(24.dp).background(Color(0x33000000)),
                    )
                else ->
                    Box(
                        modifier = stackModifier(node.style).fillMaxWidth().height(1.dp).background(Color(0x33000000)),
                    )
            }
        is ViewNode.Spacer ->
            Spacer(modifier = stackModifier(node.style).height((node.size ?: 8f).dp))
        is ViewNode.Dropzone ->
            Button(
                onClick = { node.onDrop?.let(CrepusActionState::dispatch) },
                modifier = stackModifier(node.style).fillMaxWidth(),
            ) {
                if (node.children.isEmpty()) {
                    Text(node.label)
                } else {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        node.children.forEach { child -> ViewNodeView(child) }
                    }
                }
            }
        is ViewNode.FilePicker ->
            Button(onClick = { node.onPick?.let(CrepusActionState::dispatch) }, modifier = stackModifier(node.style)) {
                Text(node.label)
            }
        is ViewNode.Image ->
            Box(
                modifier =
                    stackModifier(node.style)
                        .semantics { contentDescription = node.alt ?: node.src },
                contentAlignment = Alignment.Center,
            ) {
                Text("Unsupported remote image: ${node.alt ?: node.src}", fontSize = 12.sp, color = Color.Gray)
            }
        is ViewNode.Scroll -> {
            val scroll = rememberScrollState()
            val gap = 8.dp
            val mod =
                stackModifier(node.style).then(
                    when (node.axis) {
                        "row" -> Modifier.horizontalScroll(scroll)
                        else -> Modifier.verticalScroll(scroll)
                    },
                )
            when (node.axis) {
                "row" ->
                    Row(modifier = mod, horizontalArrangement = Arrangement.spacedBy(gap)) {
                        node.children.forEach { child -> ViewNodeView(child) }
                    }
                else ->
                    Column(modifier = mod, verticalArrangement = Arrangement.spacedBy(gap)) {
                        node.children.forEach { child -> ViewNodeView(child) }
                    }
            }
        }
        is ViewNode.ListNode ->
            Column(modifier = stackModifier(node.style), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                node.children.forEachIndexed { index, child ->
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Top) {
                        Text(if (node.ordered) "${index + 1}." else "•")
                        ViewNodeView(child)
                    }
                }
            }
        is ViewNode.ListItem ->
            Column(modifier = stackModifier(node.style), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                node.children.forEach { child -> ViewNodeView(child) }
            }
        is ViewNode.SlotRotate ->
            styledText(node.phrases.firstOrNull() ?: "", node.style)
        is ViewNode.Input -> {
            var text by remember { mutableStateOf("") }
            OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                modifier = stackModifier(node.style).fillMaxWidth(),
                placeholder = { Text(node.placeholder) },
                minLines = if (node.multiline) 4 else 1,
                maxLines = if (node.multiline) 6 else 1,
            )
        }
        is ViewNode.Picker -> {
            var selection by remember {
                mutableStateOf(node.options.firstOrNull()?.value ?: normalizedBindingValue(node.bind))
            }
            Row(
                modifier = stackModifier(node.style).fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                node.options.forEach { option ->
                    Button(
                        onClick = { selection = option.value },
                        modifier =
                            Modifier.then(
                                if (selection == option.value) {
                                    Modifier
                                } else {
                                    Modifier.border(1.dp, Color(0x33000000), RoundedCornerShape(999.dp))
                                },
                            ),
                    ) {
                        Text(option.label)
                    }
                }
            }
        }
    }
}

@Composable
private fun styledText(content: String, style: ViewStyle?) {
    val s = style
    Text(
        text = content,
        modifier = textModifier(s),
        fontSize = s?.fontSize?.sp ?: 16.sp,
        fontWeight =
            s?.fontWeight?.let { w ->
                FontWeight(w)
            },
        fontStyle = if (s?.italic == true) FontStyle.Italic else FontStyle.Normal,
        color = s?.foregroundColor?.let { parseColor(it) } ?: Color.Unspecified,
        textDecoration =
            when {
                s?.underline == true && s.strikethrough == true ->
                    TextDecoration.Underline + TextDecoration.LineThrough
                s?.underline == true -> TextDecoration.Underline
                s?.strikethrough == true -> TextDecoration.LineThrough
                else -> null
            },
        textAlign =
            when (s?.textAlign) {
                "center" -> TextAlign.Center
                "trailing" -> TextAlign.Right
                else -> TextAlign.Left
            },
    )
}

@Composable
private fun withContentColor(style: ViewStyle?, content: @Composable () -> Unit) {
    val color = style?.foregroundColor?.let { parseColor(it) }
    if (color != null) {
        CompositionLocalProvider(LocalContentColor provides color) { content() }
    } else {
        content()
    }
}

private fun textModifier(s: ViewStyle?): Modifier {
    if (s == null) return Modifier
    var m: Modifier = Modifier
    val pt = s.paddingTop ?: s.paddingVertical ?: s.padding ?: 0f
    val pb = s.paddingBottom ?: s.paddingVertical ?: s.padding ?: 0f
    val pl = s.paddingLeft ?: s.paddingHorizontal ?: s.padding ?: 0f
    val pr = s.paddingRight ?: s.paddingHorizontal ?: s.padding ?: 0f
    if (pt > 0 || pb > 0 || pl > 0 || pr > 0) {
        m = m.padding(pl.dp, pt.dp, pr.dp, pb.dp)
    }
    s.backgroundColor?.let { c ->
        m = m.background(parseColor(c), RoundedCornerShape((s.cornerRadius ?: 0f).dp))
    }
    return m
}

private fun stackModifier(s: ViewStyle?): Modifier {
    if (s == null) return Modifier
    var m: Modifier = Modifier
    m = m.applySize(s)
    val pt = s.paddingTop ?: s.paddingVertical ?: s.padding ?: 0f
    val pb = s.paddingBottom ?: s.paddingVertical ?: s.padding ?: 0f
    val pl = s.paddingLeft ?: s.paddingHorizontal ?: s.padding ?: 0f
    val pr = s.paddingRight ?: s.paddingHorizontal ?: s.padding ?: 0f
    if (pt > 0 || pb > 0 || pl > 0 || pr > 0) {
        m = m.padding(pl.dp, pt.dp, pr.dp, pb.dp)
    }
    val bg = s.backgroundColor?.let { parseColor(it) }
    val r = s.cornerRadius ?: 0f
    val shape = RoundedCornerShape(r.dp)
    s.borderWidth?.let { width ->
        m =
            m.border(
                width = width.dp,
                color = s.borderColor?.let(::parseColor) ?: Color(0x33000000),
                shape = shape,
            )
    }
    if (bg != null && r > 0) {
        m = m.clip(shape).background(bg)
    } else if (bg != null) {
        m = m.background(bg)
    }
    return m
}

private fun Modifier.applySize(s: ViewStyle): Modifier {
    var m = this
    if (s.width == -1f || s.maxWidth == -1f) {
        m = m.fillMaxWidth()
    } else if (s.width != null && s.width > 0f) {
        m = m.width(s.width.dp)
    }
    if (s.height == -1f || s.maxHeight == -1f) {
        m = m.fillMaxHeight()
    } else if (s.height != null && s.height > 0f) {
        m = m.height(s.height.dp)
    }
    return m
}

private fun columnAlign(a: String?): Alignment.Horizontal =
    when (a) {
        "end" -> Alignment.End
        "center" -> Alignment.CenterHorizontally
        "stretch" -> Alignment.CenterHorizontally
        else -> Alignment.Start
    }

private fun rowAlign(a: String?): Alignment.Vertical =
    when (a) {
        "end" -> Alignment.Bottom
        "center" -> Alignment.CenterVertically
        "stretch" -> Alignment.CenterVertically
        else -> Alignment.Top
    }

private fun parseColor(hex: String): Color {
    var t = hex.trim()
    if (t.startsWith("#")) t = t.substring(1)
    val v = t.toLong(16)
    return when (t.length) {
        6 -> Color(0xFF000000L or v)
        8 -> Color(v)
        else -> Color.Gray
    }
}

private fun badgeColor(tone: String?): Color =
    when (tone) {
        "success" -> Color(0xFF15803D)
        "warning" -> Color(0xFFD97706)
        "danger" -> Color(0xFFB91C1C)
        else -> Color(0xFF111111)
    }

private fun progressValue(value: Float, min: Float, max: Float): Float {
    val span = max - min
    if (span <= 0f) return 0f
    return ((value - min) / span).coerceIn(0f, 1f)
}

private fun sliderSteps(min: Float, max: Float, step: Float?): Int {
    val size = step ?: return 0
    if (size <= 0f) return 0
    val slots = ((max - min) / size).roundToInt()
    return (slots - 1).coerceAtLeast(0)
}

private fun quantizeSlider(value: Float, min: Float, step: Float?): Float {
    val size = step ?: return value
    if (size <= 0f) return value
    return (((value - min) / size).roundToInt() * size) + min
}

private fun normalizedBindingValue(bind: String): String {
    val trimmed = bind.trim()
    return if (trimmed.length >= 2 && trimmed.first() == '"' && trimmed.last() == '"') {
        trimmed.substring(1, trimmed.length - 1)
    } else {
        trimmed
    }
}
