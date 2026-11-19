package com.sharkord.plus

import android.app.Application
import android.os.Build
import android.os.Environment
import android.util.Log
import java.io.File
import java.io.PrintWriter
import java.io.StringWriter
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

class VibeSharkApp : Application() {
    companion object {
        private const val TAG = "VibeSharkCrash"
        const val LOG_FILENAME = "vibeshark_crash.log"
    }

    override fun onCreate() {
        super.onCreate()
        setupCrashLogging()
        preloadNativeLibraries()
    }

    private fun preloadNativeLibraries() {
        try {
            System.loadLibrary("c++_shared")
            Log.i(TAG, "Successfully preloaded libc++_shared.so")
        } catch (t: Throwable) {
            Log.w(TAG, "Notice: libc++_shared.so preload attempt: ${t.message}")
        }
    }

    private fun setupCrashLogging() {
        val defaultHandler = Thread.getDefaultUncaughtExceptionHandler()

        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            try {
                val sw = StringWriter()
                val pw = PrintWriter(sw)
                throwable.printStackTrace(pw)
                val stackTraceStr = sw.toString()

                val timeFormat = SimpleDateFormat("yyyy-MM-dd HH:mm:ss.SSS", Locale.US)
                val currentTime = timeFormat.format(Date())

                val pInfo = try {
                    packageManager.getPackageInfo(packageName, 0)
                } catch (_: Throwable) {
                    null
                }
                val verName = pInfo?.versionName ?: "unknown"
                val verCode = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                    pInfo?.longVersionCode ?: -1L
                } else {
                    @Suppress("DEPRECATION")
                    pInfo?.versionCode?.toLong() ?: -1L
                }

                val abis = Build.SUPPORTED_ABIS.joinToString(", ")

                val report = buildString {
                    appendLine("==================================================")
                    appendLine("            VIBESHARK FATAL CRASH REPORT          ")
                    appendLine("==================================================")
                    appendLine("Time: $currentTime")
                    appendLine("App Version: $verName (code: $verCode)")
                    appendLine("Device: ${Build.MANUFACTURER} ${Build.MODEL} (${Build.DEVICE})")
                    appendLine("Android OS: ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})")
                    appendLine("Fingerprint: ${Build.FINGERPRINT}")
                    appendLine("Supported ABIs: $abis")
                    appendLine("Thread: ${thread.name} (ID: ${thread.id})")
                    appendLine("Exception Type: ${throwable.javaClass.name}")
                    appendLine("Message: ${throwable.message}")
                    appendLine("--------------------------------------------------")
                    appendLine("STACK TRACE:")
                    appendLine(stackTraceStr)
                    appendLine("==================================================")
                }

                Log.e(TAG, report)

                // 1. Write to app-specific external files dir (always writable without runtime permission)
                // Path typically: /storage/emulated/0/Android/data/com.sharkord.plus/files/vibeshark_crash.log
                try {
                    val appFilesDir = getExternalFilesDir(null)
                    if (appFilesDir != null) {
                        val logFile = File(appFilesDir, LOG_FILENAME)
                        logFile.writeText(report)
                        Log.i(TAG, "Crash log saved to app storage: ${logFile.absolutePath}")
                    }
                } catch (e: Throwable) {
                    Log.e(TAG, "Failed to write crash log to externalFilesDir", e)
                }

                // 2. Also try writing to public Downloads folder for convenient access
                try {
                    val downloadsDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
                    if (downloadsDir != null && downloadsDir.exists()) {
                        val dlFile = File(downloadsDir, LOG_FILENAME)
                        dlFile.writeText(report)
                        Log.i(TAG, "Crash log saved to Downloads: ${dlFile.absolutePath}")
                    }
                } catch (_: Throwable) {
                    // Scoped storage may restrict direct public directory writes on newer Android
                }

                // 3. Write to internal filesDir as guaranteed fallback
                try {
                    val internalFile = File(filesDir, LOG_FILENAME)
                    internalFile.writeText(report)
                } catch (_: Throwable) {
                }
            } catch (fatal: Throwable) {
                Log.e(TAG, "Fatal error inside VibeShark crash handler", fatal)
            } finally {
                defaultHandler?.uncaughtException(thread, throwable)
            }
        }
    }
}
