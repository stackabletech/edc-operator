/*
 *  Copyright (c) 2022 IONOS
 *
 *  This program and the accompanying materials are made available under the
 *  terms of the Apache License, Version 2.0 which is available at
 *  https://www.apache.org/licenses/LICENSE-2.0
 *
 *  SPDX-License-Identifier: Apache-2.0
 *
 *  Contributors:
 *      IONOS
 *
 */

package com.ionos.edc.extension.s3.schema;

public interface IonosBucketSchema {
    String TYPE = "IonosS3";
    String STORAGE_NAME = "storage";
    String BUCKET_NAME = "bucketName";
    String BLOB_NAME = "blobName";
    String ACCESS_KEY_ID = "accessKey";
    String SECRET_ACCESS_KEY = "secretKey";
}
