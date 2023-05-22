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

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.fasterxml.jackson.databind.annotation.JsonPOJOBuilder;
import com.ionos.edc.extension.s3.schema.IonosBucketSchema;
import org.eclipse.edc.connector.transfer.spi.types.ProvisionedDataDestinationResource;

import static com.ionos.edc.extension.s3.schema.IonosBucketSchema.BUCKET_NAME;
import static com.ionos.edc.extension.s3.schema.IonosBucketSchema.STORAGE_NAME;

@JsonDeserialize(builder = IonosS3ProvisionedResource.Builder.class)
@JsonTypeName("dataspaceconnector:ionosprovisionedresource")
public class IonosS3ProvisionedResource extends ProvisionedDataDestinationResource {
    private String keyId;
    
    public String getStorage() {
        return getDataAddress().getProperty(STORAGE_NAME);
    }
    public String getBucketName() {
        return getDataAddress().getProperty(BUCKET_NAME);
    }
    
    
 
    public String getKeyId() {
		return keyId;
	}
	public void setKeyId(String keyId) {
		this.keyId = keyId;
	}
	@Override
    public String getResourceName() {
        return dataAddress.getProperty(BUCKET_NAME);
    }

 
    private IonosS3ProvisionedResource() {
    }

    @JsonPOJOBuilder(withPrefix = "")
    public static class Builder
            extends ProvisionedDataDestinationResource.Builder<IonosS3ProvisionedResource, Builder> {

        private Builder() {
            super(new IonosS3ProvisionedResource());
            dataAddressBuilder.type(IonosBucketSchema.TYPE);
        }

        @JsonCreator
        public static Builder newInstance() {
            return new Builder();
        }

        public Builder storage(String storage) {
            dataAddressBuilder.property(STORAGE_NAME, storage);
            return this;
        }
        public Builder bucketName(String bucketName) {
            dataAddressBuilder.property(BUCKET_NAME, bucketName);
            dataAddressBuilder.keyName(bucketName + "-key");
            return this;
        }

        public Builder keyId(String arn) {
            provisionedResource.keyId = arn;
            return this;
        }
    }
}
