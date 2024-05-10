/*
 *  Copyright (c) 2020, 2021 Microsoft Corporation
 *
 *  This program and the accompanying materials are made available under the
 *  terms of the Apache License, Version 2.0 which is available at
 *  https://www.apache.org/licenses/LICENSE-2.0
 *
 *  SPDX-License-Identifier: Apache-2.0
 *
 *  Contributors:
 *       Microsoft Corporation - initial API and implementation
 *
 */

plugins {
    `java-library`
    id("application")
    id("com.github.johnrengelman.shadow") version "7.0.0"
}
repositories {

    maven {// while runtime-metamodel dependency is still a snapshot
		url = uri("https://oss.sonatype.org/content/repositories/snapshots/")
    }
	  maven {
        url = uri("https://maven.iais.fraunhofer.de/artifactory/eis-ids-public/")
    }

	mavenLocal()
	mavenCentral()
}

val javaVersion: String by project
val edcGroup: String by project
val edcVersion: String by project
val okHttpVersion: String by project
val rsApi: String by project
val metaModelVersion: String by project
val fraunhoferVersion: String by project




dependencies {
	implementation("${edcGroup}:boot:${edcVersion}")

    implementation("${edcGroup}:control-plane-core:${edcVersion}")

	implementation("${edcGroup}:api-observability:${edcVersion}")

	implementation("${edcGroup}:configuration-filesystem:${edcVersion}")

	implementation("${edcGroup}:http:${edcVersion}")
	implementation("${edcGroup}:dsp:${edcVersion}")

	implementation("${edcGroup}:auth-tokenbased:${edcVersion}")

    implementation("$edcGroup:management-api:$edcVersion")

	//Data plane
	//implementation("${edcGroup}:data-plane-transfer-client:${edcVersion}")

	implementation("${edcGroup}:data-plane-selector-client:${edcVersion}")

	implementation("${edcGroup}:data-plane-selector-core:${edcVersion}")

	implementation("${edcGroup}:data-plane-core:${edcVersion}")

	implementation("${edcGroup}:iam-mock:${edcVersion}")

	implementation("${edcGroup}:vault-hashicorp:${edcVersion}")

	implementation("${edcGroup}:data-plane-client:${edcVersion}")

	implementation("${edcGroup}:transfer-data-plane:${edcVersion}")

	//Ionos Extension
	implementation(project(":edc-ionos-extension:provision-ionos-s3"))

	implementation(project(":edc-ionos-extension:data-plane-ionos-s3"))

	testImplementation ("${edcGroup}:junit:${edcVersion}")


	implementation("de.fraunhofer.iais.eis.ids.infomodel:java:${fraunhoferVersion}")

	//Logging
	implementation("${edcGroup}:monitor-jdk-logger:${edcVersion}")
}

repositories {
	mavenLocal()
	mavenCentral()
    maven {// while runtime-metamodel dependency is still a snapshot
		url = uri("https://oss.sonatype.org/content/repositories/snapshots/")
    }
}

application {
    mainClass.set("org.eclipse.edc.boot.system.runtime.BaseRuntime")
}

tasks.withType<com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar> {
	exclude("**/pom.properties", "**/pom.xm")
    mergeServiceFiles()
    archiveFileName.set("dataspace-connector.jar")
}
