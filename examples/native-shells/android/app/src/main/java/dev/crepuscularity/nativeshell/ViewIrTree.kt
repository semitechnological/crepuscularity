package dev.crepuscularity.nativeshell

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun ViewIrRoot(ir: ViewIr, modifier: Modifier = Modifier) {
    Column(modifier = modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(0.dp)) {
        ir.root.forEach { node -> ViewNodeView(node) }
    }
}

@Composable
fun ViewNodeView(node: ViewNode) {
    when (node) {
        is ViewNode.Text -> Text(text = node.content)
        is ViewNode.Stack -> {
            val gap = (node.spacing ?: 8f).dp
            val spaced = Arrangement.spacedBy(gap)
            when (node.axis) {
                "row" ->
                    Row(horizontalArrangement = spaced) {
                        node.children.forEach { child -> ViewNodeView(child) }
                    }

                else ->
                    Column(verticalArrangement = spaced) {
                        node.children.forEach { child -> ViewNodeView(child) }
                    }
            }
        }
    }
}
