# VoidCaller ProGuard Rules

# Keep OkHttp
-dontwarn okhttp3.**
-dontwarn okio.**
-keep class okhttp3.** { *; }

# Keep Moshi
-keep class com.squareup.moshi.** { *; }

# Keep Shizuku
-keep class rikka.shizuku.** { *; }

# Keep Compose
-keep class androidx.compose.** { *; }
