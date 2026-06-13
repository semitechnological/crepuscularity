package dev.crepuscularity.nativeshell

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun ViewIrRoot(ir: ViewIr, modifier: Modifier = Modifier) {
    Column(modifier = modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(0.dp)) {
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
                    Row(
                        modifier = mod,
                        horizontalArrangement = arr,
                        verticalAlignment = rowAlign,
                    ) {
                        node.children.forEach { child -> ViewNodeView(child) }
                    }
                else ->
                    Column(
                        modifier = mod,
                        verticalArrangement = arr,
                        horizontalAlignment = colAlign,
                    ) {
                        node.children.forEach { child -> ViewNodeView(child) }
                    }
            }
        }
        is ViewNode.Button ->
            Button(onClick = { /* shell demo: ${node.onClick} */ }) {
                Text(node.label)
            }
        is ViewNode.Image ->
            Column(modifier = stackModifier(node.style)) {
                Text(node.alt ?: node.src, fontSize = 12.sp, color = Color.Gray)
                Text("[image: ${node.src}]", fontSize = 10.sp)
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
        is ViewNode.SlotRotate ->
            styledText(node.phrases.firstOrNull() ?: "", node.style)
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
    val pt = s.paddingTop ?: s.paddingVertical ?: s.padding ?: 0f
    val pb = s.paddingBottom ?: s.paddingVertical ?: s.padding ?: 0f
    val pl = s.paddingLeft ?: s.paddingHorizontal ?: s.padding ?: 0f
    val pr = s.paddingRight ?: s.paddingHorizontal ?: s.padding ?: 0f
    if (pt > 0 || pb > 0 || pl > 0 || pr > 0) {
        m = m.padding(pl.dp, pt.dp, pr.dp, pb.dp)
    }
    val bg = s.backgroundColor?.let { parseColor(it) }
    val r = s.cornerRadius ?: 0f
    if (bg != null && r > 0) {
        m = m.clip(RoundedCornerShape(r.dp)).background(bg)
    } else if (bg != null) {
        m = m.background(bg)
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
