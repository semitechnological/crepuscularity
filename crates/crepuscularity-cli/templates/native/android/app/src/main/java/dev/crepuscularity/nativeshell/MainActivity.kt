package dev.crepuscularity.nativeshell

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            val runtime = remember { CrepusMobileRuntime(this) }
            LaunchedEffect(runtime) { runtime.start() }
            MaterialTheme {
                val ir = runtime.ir.value
                val error = runtime.errorText.value
                when {
                    ir != null -> ViewIrRoot(ir = ir)
                    error != null -> Text(error)
                    else -> Text("Loading")
                }
            }
        }
    }
}
