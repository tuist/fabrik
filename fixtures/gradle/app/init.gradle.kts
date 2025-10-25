// Configure remote build cache
// The URL will be set dynamically by the fabrik wrapper via -D system property
gradle.settingsEvaluated {
    buildCache {
        remote<HttpBuildCache> {
            // Read URL from system property set by fabrik wrapper
            val cacheUrl = System.getProperty("org.gradle.caching.buildCache.remote.url")
            if (cacheUrl != null) {
                url = uri(cacheUrl)
                isPush = System.getProperty("org.gradle.caching.buildCache.remote.push")?.toBoolean() ?: true
            }
        }
    }
}
