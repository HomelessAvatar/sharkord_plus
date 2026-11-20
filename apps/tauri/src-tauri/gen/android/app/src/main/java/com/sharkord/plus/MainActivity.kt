package com.sharkord.plus

import android.os.Bundle
import android.webkit.PermissionRequest
import android.webkit.WebChromeClient
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  companion object {
    @JvmStatic
    var instance: MainActivity? = null

    init {
      try {
        System.loadLibrary("c++_shared")
      } catch (_: Throwable) {
      }
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    try {
      enableEdgeToEdge()
    } catch (_: Throwable) {
      // Graceful fallback for OEM skins (e.g. Xiaomi HyperOS/MIUI) with custom window insets
    }
    super.onCreate(savedInstanceState)
    instance = this

    findViewById<android.view.View>(android.R.id.content)?.let { contentView ->
      ViewCompat.setOnApplyWindowInsetsListener(contentView) { view, insets ->
        val bars = insets.getInsets(
          WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
        )
        view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
        insets
      }
      ViewCompat.requestApplyInsets(contentView)
    }
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
