package dev.crepuscularity.nativeshell

import android.content.pm.ApplicationInfo
import android.os.Bundle
import androidx.activity.SystemBarStyle
import androidx.activity.ComponentActivity
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge(
            statusBarStyle = SystemBarStyle.dark(android.graphics.Color.TRANSPARENT),
            navigationBarStyle = SystemBarStyle.dark(android.graphics.Color.TRANSPARENT),
        )
        super.onCreate(savedInstanceState)
        CrepusRustActions.install(this)
        setContent {
            MaterialTheme {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(Color(0xFF101624))
                        .safeDrawingPadding()
                ) {
                    if ((applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0) {
                        val runtime = remember {
                            CrepusMobileRuntime(this@MainActivity).also { it.start() }
                        }
                        val ir = runtime.ir.value
                        val error = runtime.errorText.value
                        when {
                            ir != null -> ViewIrRoot(ir, modifier = Modifier.fillMaxSize())
                            error != null -> Text(error)
                            else -> Text("")
                        }
                    } else {
                        CrepusGeneratedView(modifier = Modifier.fillMaxSize())
                    }
                }
            }
        }
    }
}
