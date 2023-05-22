plugins {
	`java-library`

}
repositories {
	mavenLocal()
	mavenCentral()
    maven {// while runtime-metamodel dependency is still a snapshot
		url = uri("https://oss.sonatype.org/content/repositories/snapshots/")
    }
}

val javaVersion: String by project
val faaastVersion: String by project
val edcGroup: String by project
val edcVersion: String by project
val okHttpVersion: String by project
val rsApi: String by project
val metaModelVersion: String by project
val minIOVersion: String by project

dependencies {

	api("${edcGroup}:runtime-metamodel:${metaModelVersion}")
	implementation("${edcGroup}:transfer-spi:${edcVersion}")

	//implementation("${edcGroup}:vault-hashicorp:${edcVersion}")
	
	implementation("io.minio:minio:${minIOVersion}")
	
	testImplementation ("${edcGroup}:junit:${edcVersion}")	
	

}
