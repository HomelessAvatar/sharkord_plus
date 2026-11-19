# Tauri Core & JNI Bridge
-keep class app.tauri.** { *; }
-dontwarn app.tauri.**

# Sharkord Plus application classes and services
-keep class com.sharkord.plus.** { *; }
-dontwarn com.sharkord.plus.**

# AndroidX WebKit, Activity, and AppCompat
-keep class androidx.webkit.** { *; }
-dontwarn androidx.webkit.**
-keep class androidx.activity.** { *; }
-dontwarn androidx.activity.**
-keep class androidx.appcompat.** { *; }
-dontwarn androidx.appcompat.**

# Preserve annotations for Tauri IPC and plugin reflection
-keepattributes *Annotation*, Signature, InnerClasses, EnclosingMethod
-keep class app.tauri.annotation.** { *; }
-keep class * implements app.tauri.plugin.Plugin { *; }

-keepclassmembers class * {
    @app.tauri.annotation.Command <methods>;
    @app.tauri.annotation.InvokeArg <fields>;
    @app.tauri.annotation.InvokeArg <methods>;
}
