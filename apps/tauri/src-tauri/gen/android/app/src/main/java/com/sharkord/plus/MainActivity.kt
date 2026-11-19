package com.sharkord.plus

import android.os.Bundle
import android.webkit.PermissionRequest
import android.webkit.WebChromeClient
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  companion object {
    @JvmStatic
    var instance: MainActivity? = null
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    try {
      enableEdgeToEdge()
    } catch (_: Throwable) {
      // Graceful fallback for OEM skins (e.g. Xiaomi HyperOS/MIUI) with custom window insets
    }
    super.onCreate(savedInstanceState)
    instance = this
  }

  override fun onDestroy() {
    super.onDestroy()
    if (instance == this) {
      instance = null
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    webView.webChromeClient = object : WebChromeClient() {
      override fun onPermissionRequest(request: PermissionRequest) {
        val origin = request.origin.toString()
        if (origin.startsWith("tauri://localhost") ||
            origin.startsWith("https://tauri.localhost") ||
            origin.startsWith("http://localhost")) {
          val grantedResources = request.resources.filter {
            it == PermissionRequest.RESOURCE_AUDIO_CAPTURE || it == PermissionRequest.RESOURCE_VIDEO_CAPTURE
          }.toTypedArray()

          if (grantedResources.isNotEmpty()) {
            request.grant(grantedResources)
            return
          }
        }
        super.onPermissionRequest(request)
      }
    }
  }

  fun setVoiceActive(active: Boolean) {
    if (active) {
      VoiceForegroundService.start(this)
    } else {
      VoiceForegroundService.stop(this)
    }
  }
}
