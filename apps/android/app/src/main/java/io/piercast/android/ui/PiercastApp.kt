package io.piercast.android.ui

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import io.piercast.android.data.AppInfo
import io.piercast.android.data.PairedHost
import io.piercast.android.data.PiercastApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
fun PiercastApp() {
    val nav = rememberNavController()
    var host by remember { mutableStateOf<PairedHost?>(null) }
    var apps by remember { mutableStateOf(listOf<AppInfo>()) }
    val scope = rememberCoroutineScope()

    fun refresh() {
        val h = host ?: return
        scope.launch {
            apps = withContext(Dispatchers.IO) { PiercastApi.listApps(h) }
        }
    }

    NavHost(navController = nav, startDestination = "home") {
        composable("home") {
            Scaffold(
                topBar = { TopAppBar(title = { Text("Piercast") }) },
                floatingActionButton = {
                    FloatingActionButton(onClick = { nav.navigate("settings") }) { Text("⚙") }
                }
            ) { pad ->
                if (host == null) {
                    Text(
                        "Pair a desktop via Settings → scan QR",
                        modifier = Modifier.padding(pad).padding(16.dp)
                    )
                } else {
                    LazyColumn(Modifier = Modifier.padding(pad)) {
                        items(apps, key = { it.id }) { app ->
                            ListItem(
                                headlineContent = { Text(app.name) },
                                supportingContent = { Text(app.status) },
                                modifier = Modifier
                                    .semantics { contentDescription = "${app.name}, ${app.status}" }
                                    .clickable { nav.navigate("app/${app.id}") }
                            )
                            HorizontalDivider()
                        }
                    }
                    LaunchedEffect(host) { refresh() }
                }
            }
        }
        composable("app/{id}") { entry ->
            val id = entry.arguments?.getString("id")
            val app = apps.firstOrNull { it.id == id }
            val context = LocalContext.current
            Scaffold(topBar = { TopAppBar(title = { Text(app?.name ?: "App") }) }) { pad ->
                LazyColumn(modifier = Modifier.padding(pad).padding(16.dp)) {
                    item {
                        Text("Status: ${app?.status ?: "?"}")
                        Button(onClick = {
                            val h = host ?: return@Button
                            scope.launch {
                                withContext(Dispatchers.IO) { PiercastApi.post(h, "v1/apps/$id/open") }
                                val url = app?.expose_url ?: app?.open_url
                                if (url != null) {
                                    context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                                }
                                refresh()
                            }
                        }) { Text("Open") }
                        Button(onClick = {
                            val h = host ?: return@Button
                            scope.launch {
                                withContext(Dispatchers.IO) { PiercastApi.post(h, "v1/apps/$id/start", """{"mode":"development"}""") }
                                refresh()
                            }
                        }) { Text("Start") }
                        Button(onClick = {
                            val h = host ?: return@Button
                            scope.launch {
                                withContext(Dispatchers.IO) { PiercastApi.post(h, "v1/apps/$id/stop") }
                                refresh()
                            }
                        }) { Text("Stop") }
                        Button(onClick = {
                            val h = host ?: return@Button
                            scope.launch {
                                withContext(Dispatchers.IO) { PiercastApi.post(h, "v1/apps/$id/restart") }
                                refresh()
                            }
                        }) { Text("Restart") }
                        Button(onClick = {
                            val h = host ?: return@Button
                            scope.launch {
                                withContext(Dispatchers.IO) { PiercastApi.post(h, "v1/apps/$id/kill") }
                                refresh()
                            }
                        }) { Text("Kill") }
                    }
                }
            }
        }
        composable("settings") {
            Scaffold(topBar = { TopAppBar(title = { Text("Settings") }) }) { pad ->
                Text(
                    "QR pairing: decode bootstrap JSON from desktop POST /v1/pair/complete. Hosts stored in app preferences.",
                    modifier = Modifier.padding(pad).padding(16.dp)
                )
            }
        }
    }
}
