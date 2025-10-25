plugins {
    kotlin("jvm") version "2.1.0"
    application
}

repositories {
    mavenCentral()
}

application {
    mainClass.set("com.example.AppKt")
}

// Don't specify jvmToolchain - use whatever Java version is available
// This makes the fixture work across different CI environments
