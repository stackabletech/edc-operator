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
 *       Fraunhofer Institute for Software and Systems Engineering - added dependencies
 *
 */

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


dependencies {


	api("${edcGroup}:data-plane-spi:${edcVersion}")
	implementation("${edcGroup}:transfer-spi:${edcVersion}")
	implementation("${edcGroup}:data-plane-util:${edcVersion}")
	implementation(project(":edc-ionos-extension:core-ionos-s3"))

    implementation("${edcGroup}:data-plane-core:${edcVersion}")
	testImplementation("${edcGroup}:data-plane-core:${edcVersion}")

	implementation("${edcGroup}:http:${edcVersion}")

	testImplementation("org.junit.jupiter:junit-jupiter-api:5.10.2")

    testImplementation("org.assertj:assertj-core:3.22.0")
    implementation("org.realityforge.org.jetbrains.annotations:org.jetbrains.annotations:1.7.0")

}
