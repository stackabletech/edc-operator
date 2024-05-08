plugins {
    `java-library`
}
val javaVersion: String by project
val faaastVersion: String by project
val edcGroup: String by project
val edcVersion: String by project
val okHttpVersion: String by project
val rsApi: String by project
val metaModelVersion: String by project

repositories {
	mavenLocal()
	mavenCentral()
    maven {// while runtime-metamodel dependency is still a snapshot
		url = uri("https://oss.sonatype.org/content/repositories/snapshots/")
    }
}
dependencies {


	api("${edcGroup}:runtime-metamodel:${metaModelVersion}")
    implementation(project(":edc-ionos-extension:core-ionos-s3"))
	implementation("${edcGroup}:transfer-spi:${edcVersion}")

	testImplementation("org.junit.jupiter:junit-jupiter-api:5.9.1")

	implementation("dev.failsafe:failsafe:3.2.4")
}
