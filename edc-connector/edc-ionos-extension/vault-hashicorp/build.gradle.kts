/*
 *  Copyright (c) 2022 Mercedes-Benz Tech Innovation GmbH
 *
 *  This program and the accompanying materials are made available under the
 *  terms of the Apache License, Version 2.0 which is available at
 *  https://www.apache.org/licenses/LICENSE-2.0
 *
 *  SPDX-License-Identifier: Apache-2.0
 *
 *  Contributors:
 *       Mercedes-Benz Tech Innovation GmbH - Initial API and Implementation
 *
 */
plugins {
    `java-library`
}

val edcGroup: String by project
val edcVersion: String by project

dependencies {
    api("${edcGroup}:core-spi:${edcVersion}")
    api("${edcGroup}:http-spi:${edcVersion}")

    implementation("${edcGroup}:util:${edcVersion}")
}

