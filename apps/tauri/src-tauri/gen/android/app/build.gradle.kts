import java.util.Properties
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

android {
    compileSdk = 37
    namespace = "com.sharkord.plus"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "true"
        applicationId = "com.sharkord.plus"
        minSdk = 24
        targetSdk = 34
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    packaging {
        jniLibs {
            useLegacyPackaging = false
        }
    }
    signingConfigs {
        create("release") {
            val keyAliasEnv = System.getenv("ANDROID_KEY_ALIAS")
            val keyPasswordEnv = System.getenv("ANDROID_KEY_PASSWORD")
            val storeFileEnv = System.getenv("ANDROID_KEYSTORE_PATH")
            val storePasswordEnv = System.getenv("ANDROID_STORE_PASSWORD")

            if (storeFileEnv != null && file(storeFileEnv).exists()) {
                storeFile = file(storeFileEnv)
                storePassword = storePasswordEnv ?: "sharkord"
                keyAlias = keyAliasEnv ?: "sharkord"
                keyPassword = keyPasswordEnv ?: "sharkord"
            } else {
                // Fall back to debug signing config so release builds are always signed and installable
                initWith(getByName("debug"))
            }
        }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {
                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    buildFeatures {
        buildConfig = true
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_1_8
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.lifecycle:lifecycle-process:2.10.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = file("tauri.build.gradle.kts"))

tasks.register("copyNdkSharedLibs") {
    doFirst {
        val envNdk = System.getenv("NDK_HOME") ?: System.getenv("ANDROID_NDK_HOME")
        val ndkDir = if (!envNdk.isNullOrBlank()) file(envNdk) else (android.ndkDirectory ?: file(""))
        if (ndkDir.exists()) {
            val abiMap = mapOf(
                "arm64-v8a" to "aarch64-linux-android",
                "armeabi-v7a" to "arm-linux-androideabi",
                "x86" to "i686-linux-android",
                "x86_64" to "x86_64-linux-android"
            )
            val prebuiltDir = file("${ndkDir}/toolchains/llvm/prebuilt")
            val hostDir = prebuiltDir.listFiles()?.firstOrNull()
            val llvmLibDir = hostDir?.resolve("sysroot/usr/lib")
            if (llvmLibDir != null && llvmLibDir.exists()) {
                abiMap.forEach { (abi, triple) ->
                    val srcLib = File(llvmLibDir, "$triple/libc++_shared.so")
                    if (srcLib.exists()) {
                        val destDir = file("src/main/jniLibs/$abi")
                        destDir.mkdirs()
                        srcLib.copyTo(File(destDir, "libc++_shared.so"), overwrite = true)
                    }
                }
            }
        }
    }
}

tasks.matching { it.name.startsWith("merge") && it.name.contains("JniLibFolders") }.configureEach {
    dependsOn("copyNdkSharedLibs")
}
