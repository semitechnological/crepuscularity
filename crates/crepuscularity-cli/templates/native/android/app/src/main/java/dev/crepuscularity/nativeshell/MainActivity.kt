package dev.crepuscularity.nativeshell

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val jsonText =
            assets.open("fixture.json").bufferedReader().use { it.readText() }
        val ir = decodeViewIr(jsonText)
        setContent {
            MaterialTheme { ViewIrRoot(ir = ir) }
        }
    }
}
