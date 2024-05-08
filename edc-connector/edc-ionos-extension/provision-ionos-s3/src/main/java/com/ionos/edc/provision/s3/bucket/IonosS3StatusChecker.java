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

package com.ionos.edc.provision.s3.bucket;

import com.ionos.edc.extension.s3.api.S3ConnectorApi;
import com.ionos.edc.extension.s3.schema.IonosBucketSchema;
import dev.failsafe.RetryPolicy;
import org.eclipse.edc.connector.transfer.spi.types.ProvisionedResource;
import org.eclipse.edc.connector.transfer.spi.types.StatusChecker;
import org.eclipse.edc.connector.transfer.spi.types.TransferProcess;
import org.eclipse.edc.spi.EdcException;

import java.util.List;

import static java.lang.String.format;

public class IonosS3StatusChecker implements StatusChecker {
    private final S3ConnectorApi s3Api;
    private final RetryPolicy<Object> retryPolicy;

    public IonosS3StatusChecker(S3ConnectorApi s3Api, RetryPolicy<Object> retryPolicy) {
        this.s3Api = s3Api;
        this.retryPolicy = retryPolicy;
    }

    @Override
    public boolean isComplete(TransferProcess transferProcess, List<ProvisionedResource> resources) {

        if (resources.isEmpty()) {
            var destination = transferProcess.getDataRequest().getDataDestination();
            var bucketName = destination.getProperty(IonosBucketSchema.BUCKET_NAME);
            return checkBucket(bucketName);
        } else {

            for (var resource : resources) {
                var provisionedResource = (IonosS3ProvisionedResource) resource;
                var bucketName = provisionedResource.getBucketName();
                return checkBucket(bucketName);
            }

        }
        throw new EdcException(
                format("No bucket resource was associated with the transfer process: %s - cannot determine completion.",
                        transferProcess.getId()));

    }

    private boolean checkBucket(String bucketName) {
        if (!s3Api.bucketExists(bucketName)) {
            return false;
        }
        return true;

    }

}
