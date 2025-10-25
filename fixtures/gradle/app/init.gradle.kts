// Configure remote build cache
// The URL will be set dynamically by the fabrik wrapper via -D system property
gradle.settingsEvaluated {
    buildCache {
        remote<HttpBuildCache> {
            // URL is set via -Dorg.gradle.caching.buildCache.remote.url
            isPush = System.getProperty("org.gradle.caching.buildCache.remote.push")?.toBoolean() ?: true
        }
    }
}
