package io.piercast.android.data

import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.concurrent.TimeUnit

data class PairedHost(val baseUrl: String, val token: String, val hostName: String)
data class AppInfo(val id: String, val name: String, val status: String, val open_url: String? = null, val expose_url: String? = null)
data class BootstrapPayload(val base_url: String, val token: String, val host_name: String, val expires_at: String)

object PiercastApi {
    private val http = OkHttpClient.Builder()
        .callTimeout(10, TimeUnit.SECONDS)
        .build()
    private val gson = Gson()
    private val json = "application/json".toMediaType()

    fun listApps(host: PairedHost): List<AppInfo> {
        val req = Request.Builder()
            .url("${host.baseUrl.trimEnd('/')}/v1/apps")
            .header("Authorization", "Bearer ${host.token}")
            .get()
            .build()
        http.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) return emptyList()
            val body = resp.body?.string().orEmpty()
            val type = object : TypeToken<List<AppInfo>>() {}.type
            return gson.fromJson(body, type)
        }
    }

    fun post(host: PairedHost, path: String, body: String? = null) {
        val req = Request.Builder()
            .url("${host.baseUrl.trimEnd('/')}/$path")
            .header("Authorization", "Bearer ${host.token}")
            .post((body ?: "{}").toRequestBody(json))
            .build()
        http.newCall(req).execute().close()
    }

    fun completePair(payload: BootstrapPayload, deviceName: String): PairedHost {
        val body = gson.toJson(mapOf("device_name" to deviceName, "platform" to "android"))
        val req = Request.Builder()
            .url("${payload.base_url.trimEnd('/')}/v1/pair/complete")
            .header("Authorization", "Bearer ${payload.token}")
            .post(body.toRequestBody(json))
            .build()
        http.newCall(req).execute().use { resp ->
            val text = resp.body?.string().orEmpty()
            val token = runCatching {
                gson.fromJson(text, Map::class.java)["token"] as? String
            }.getOrNull() ?: payload.token
            return PairedHost(payload.base_url, token, payload.host_name)
        }
    }

    fun isAllowedCleartextHost(host: String): Boolean {
        if (host == "localhost" || host.endsWith(".ts.net")) return true
        val ip = host.removePrefix("[").removeSuffix("]")
        return ip.startsWith("10.") ||
            ip.startsWith("192.168.") ||
            ip.startsWith("172.16.") ||
            ip.startsWith("172.17.") ||
            ip.startsWith("172.18.") ||
            ip.startsWith("172.19.") ||
            ip.startsWith("172.2") ||
            ip.startsWith("172.3") ||
            ip.startsWith("100.") // Tailscale CGNAT
    }
}
