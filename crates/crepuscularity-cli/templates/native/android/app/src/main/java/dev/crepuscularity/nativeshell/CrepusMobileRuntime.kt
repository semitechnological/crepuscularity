package dev.crepuscularity.nativeshell

import android.content.Context
import android.content.pm.ApplicationInfo
import android.os.Handler
import android.os.Looper
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import java.net.HttpURLConnection
import java.net.URL
import kotlin.concurrent.thread

class CrepusMobileRuntime(
    private val context: Context,
    private val baseUrl: String = "http://10.0.2.2:4001",
) {
    val ir: MutableState<ViewIr?> = mutableStateOf(null)
    val errorText: MutableState<String?> = mutableStateOf(null)
    private val mainHandler = Handler(Looper.getMainLooper())

    fun start() {
        loadFixture()
        if (
            (context.applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0 &&
            System.getenv("CREPUS_DEV_SERVER") == "1"
        ) {
            thread(name = "crepus-mobile-runtime", isDaemon = true) {
                while (!Thread.currentThread().isInterrupted) {
                    refreshFromDevServer()
                    Thread.sleep(1000)
                }
            }
        }
    }

    private fun loadFixture() {
        runCatching {
            context.assets.open("fixture.json").bufferedReader().use { it.readText() }
        }.mapCatching { jsonText ->
            decodeViewIr(jsonText)
        }.onSuccess { loaded ->
            publish(loaded, null)
        }.onFailure { error ->
            publish(null, error.message ?: error.toString())
        }
    }

    private fun refreshFromDevServer() {
        runCatching {
            val connection = URL("$baseUrl/ir").openConnection() as HttpURLConnection
            connection.connectTimeout = 500
            connection.readTimeout = 1000
            connection.inputStream.bufferedReader().use { it.readText() }
        }.mapCatching { jsonText ->
            decodeViewIr(jsonText)
        }.onSuccess { loaded ->
            publish(loaded, null)
        }
    }

    private fun publish(nextIr: ViewIr?, nextError: String?) {
        mainHandler.post {
            if (nextIr != null) {
                ir.value = nextIr
            }
            errorText.value = nextError
        }
    }
}
