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

package com.ionos.edc.dataplane.ionos.s3;


import com.ionos.edc.dataplane.ionos.s3.validation.IonosSinkDataAddressValidationRule;
import com.ionos.edc.extension.s3.api.S3ConnectorApi;
import com.ionos.edc.extension.s3.api.S3ConnectorApiImpl;
import com.ionos.edc.extension.s3.configuration.IonosToken;
import com.ionos.edc.extension.s3.schema.IonosBucketSchema;
import org.eclipse.edc.connector.dataplane.spi.pipeline.DataSink;
import org.eclipse.edc.connector.dataplane.spi.pipeline.DataSinkFactory;
import org.eclipse.edc.connector.dataplane.util.validation.ValidationRule;
import org.eclipse.edc.spi.EdcException;
import org.eclipse.edc.spi.monitor.Monitor;
import org.eclipse.edc.spi.result.Result;
import org.eclipse.edc.spi.security.Vault;
import org.eclipse.edc.spi.types.TypeManager;
import org.eclipse.edc.spi.types.domain.DataAddress;
import org.eclipse.edc.spi.types.domain.transfer.DataFlowRequest;
import org.jetbrains.annotations.NotNull;

import java.util.concurrent.ExecutorService;

import static com.ionos.edc.extension.s3.schema.IonosBucketSchema.ACCESS_KEY_ID;
import static com.ionos.edc.extension.s3.schema.IonosBucketSchema.SECRET_ACCESS_KEY;
public class IonosDataSinkFactory implements DataSinkFactory {

    private static final int CHUNK_SIZE_IN_BYTES = 1024 * 1024 * 500; // 500MB chunk size
    private static final String DEFAULT_STORAGE = "s3-eu-central-1.ionoscloud.com";

    private final ExecutorService executorService;
    private final Monitor monitor;
    private S3ConnectorApi s3Api;

    private Vault vault;
    private TypeManager typeManager;

    private final ValidationRule<DataAddress> validation = new IonosSinkDataAddressValidationRule();

    public IonosDataSinkFactory(S3ConnectorApi s3Api, ExecutorService executorService, Monitor monitor,
            Vault vault, TypeManager typeManager) {
        this.s3Api = s3Api;
        this.executorService = executorService;
        this.monitor = monitor;
        this.vault = vault;
        this.typeManager = typeManager;
    }

    @Override
    public boolean canHandle(DataFlowRequest request) {
        return true;
    }

    @Override
    public @NotNull Result<Boolean> validate(DataFlowRequest request) {
        var destination = request.getDestinationDataAddress();
        return validation.apply(destination).map(it -> true);
    }

    @Override
    public DataSink createSink(DataFlowRequest request) {
        var validationResult = validate(request);
        if (validationResult.failed()) {
            throw new EdcException(String.join(", ", validationResult.getFailureMessages()));
        }
        var destination = request.getDestinationDataAddress();

        var secret = vault.resolveSecret(destination.getKeyName());

        S3ConnectorApi s3ApiTemp;
        if (secret != null) {
            var Token = typeManager.readValue(secret, IonosToken.class);

            if (destination.getProperty(IonosBucketSchema.STORAGE_NAME)!=null) {

            s3ApiTemp = new S3ConnectorApiImpl(destination.getProperty(IonosBucketSchema.STORAGE_NAME), Token.getAccessKey(), Token.getSecretKey(), "");
            		 return IonosDataSink.Builder.newInstance().bucketName(destination.getProperty(IonosBucketSchema.BUCKET_NAME))
            	                .blobName(destination.getKeyName()).requestId(request.getId()).executorService(executorService)
            	                .monitor(monitor).s3Api(s3ApiTemp).build();
            }	 else {
            	 s3ApiTemp = new S3ConnectorApiImpl(DEFAULT_STORAGE, Token.getAccessKey(), Token.getSecretKey(), "");
            	 return IonosDataSink.Builder.newInstance().bucketName(destination.getProperty(IonosBucketSchema.BUCKET_NAME))
				   	                .blobName(destination.getKeyName()).requestId(request.getId()).executorService(executorService)
				   	                .monitor(monitor).s3Api(s3ApiTemp).build();
            }
        }

        return IonosDataSink.Builder.newInstance().bucketName(destination.getProperty(IonosBucketSchema.BUCKET_NAME))
                .blobName(destination.getKeyName()).requestId(request.getId()).executorService(executorService)
                .monitor(monitor).s3Api(s3Api).build();
    }

}
