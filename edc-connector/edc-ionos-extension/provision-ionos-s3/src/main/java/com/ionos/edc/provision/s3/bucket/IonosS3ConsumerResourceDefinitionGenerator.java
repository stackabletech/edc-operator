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

import static java.util.UUID.randomUUID;

import java.util.Objects;

import org.eclipse.edc.connector.transfer.spi.provision.ConsumerResourceDefinitionGenerator;
import org.eclipse.edc.connector.transfer.spi.types.DataRequest;
import org.eclipse.edc.connector.transfer.spi.types.ResourceDefinition;
import org.eclipse.edc.policy.model.Policy;



import com.ionos.edc.extension.s3.schema.IonosBucketSchema;

public class IonosS3ConsumerResourceDefinitionGenerator implements ConsumerResourceDefinitionGenerator {

    @Override
    public ResourceDefinition generate(DataRequest dataRequest, Policy policy) {
        Objects.requireNonNull(dataRequest, "dataRequest must always be provided");
        Objects.requireNonNull(policy, "policy must always be provided");
     
        var destination = dataRequest.getDataDestination();
        var id = randomUUID().toString();
        var accessKey = destination.getProperty(IonosBucketSchema.ACCESS_KEY_ID);
        var secretKey = destination.getProperty(IonosBucketSchema.SECRET_ACCESS_KEY);
        var bucket = destination.getProperty(IonosBucketSchema.BUCKET_NAME);
        var acc = destination.getProperty("account");
        return IonosS3ResourceDefinition.Builder.newInstance().id(id).accessKey(accessKey).secretKey(secretKey).bucketName(bucket).build();

    }

    @Override
    public boolean canGenerate(DataRequest dataRequest, Policy policy) {
        Objects.requireNonNull(dataRequest, "dataRequest must always be provided");
        Objects.requireNonNull(policy, "policy must always be provided");

        return IonosBucketSchema.TYPE.equals(dataRequest.getDestinationType());
    }

}
