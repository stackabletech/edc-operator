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

import org.eclipse.edc.connector.transfer.spi.types.ResourceDefinition;


public class IonosS3ResourceDefinition extends ResourceDefinition {
    private String storage;
    private String accessKey;
    private String secretKey;
    private String bucketName;


    public IonosS3ResourceDefinition() {
        super();
    }

    public String getStorage() {
        return storage;
    }

    public void setStorage(String storage) {
        this.storage = storage;
    }

    public String getAccessKey() {
		return accessKey;
	}



	public void setAccessKey(String accessKey) {
		this.accessKey = accessKey;
	}



	public String getSecretKey() {
		return secretKey;
	}



	public void setSecretKey(String secretKey) {
		this.secretKey = secretKey;
	}


	public String getbucketName() {
        return bucketName;
    }

    public void setbucketName(String bucketName) {
        this.bucketName = bucketName;
    }

    @Override
    public Builder toBuilder() {
        return initializeBuilder(new Builder()).storage(storage).accessKey(accessKey).secretKey(secretKey).bucketName(bucketName);
    }

    public static class Builder extends ResourceDefinition.Builder<IonosS3ResourceDefinition, Builder> {

        private Builder() {
            super(new IonosS3ResourceDefinition());
        }

        public static Builder newInstance() {
            return new Builder();
        }

        public Builder storage(String storage) {
            resourceDefinition.storage = storage;
            return this;
        }

        public Builder accessKey(String accessKey) {
            resourceDefinition.accessKey = accessKey;
            return this;
        }

        public Builder secretKey(String secretKey) {
            resourceDefinition.secretKey = secretKey;
            return this;
        }

        public Builder bucketName(String bucketName) {
            resourceDefinition.bucketName = bucketName;
            return this;
        }



        @Override
        protected void verify() {
            super.verify();

        }
    }

}
