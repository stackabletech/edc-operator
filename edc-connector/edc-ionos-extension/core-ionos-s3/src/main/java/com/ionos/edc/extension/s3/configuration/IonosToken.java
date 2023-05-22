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

package com.ionos.edc.extension.s3.configuration;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonTypeName;

import org.eclipse.edc.connector.transfer.spi.types.SecretToken;

@JsonTypeName("dataspaceconnector:ionostoken")
public class IonosToken implements SecretToken {

    private final String accessKey;
    private final String secretKey;
    private final long expiration;

    public IonosToken(@JsonProperty("accessKey") String accessKey, @JsonProperty("secretKey") String secretKey, @JsonProperty("expiration") long expiration) {
    	this.expiration = expiration;
        this.accessKey = accessKey;
        this.secretKey = secretKey;
    }
   
	public String getAccessKey() {
		return accessKey;
	}

	public String getSecretKey() {
		return secretKey;
	}
	
	@Override
	public long getExpiration() {
		return expiration;
	}

	

}
