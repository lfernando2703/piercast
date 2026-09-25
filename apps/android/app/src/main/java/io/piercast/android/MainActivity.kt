package io.piercast.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.ui.graphics.Color
import io.piercast.android.ui.PiercastApp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(
                colorScheme = darkColorScheme(
                    primary = Color(0xFF7B6CFF),
                    background = Color(0xFF0B0B0F),
                    surface = Color(0xFF0B0B0F)
                )
            ) {
                PiercastApp()
            }
        }
    }
}
