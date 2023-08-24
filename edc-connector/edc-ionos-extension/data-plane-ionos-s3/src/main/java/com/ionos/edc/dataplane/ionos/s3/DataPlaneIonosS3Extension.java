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

import com.ionos.edc.extension.s3.api.S3ConnectorApi;
import org.eclipse.edc.connector.dataplane.spi.pipeline.DataTransferExecutorServiceContainer;
import org.eclipse.edc.connector.dataplane.spi.pipeline.PipelineService;
import org.eclipse.edc.runtime.metamodel.annotation.Extension;
import org.eclipse.edc.runtime.metamodel.annotation.Inject;
import org.eclipse.edc.spi.security.Vault;
import org.eclipse.edc.spi.system.ServiceExtension;
import org.eclipse.edc.spi.system.ServiceExtensionContext;
import org.eclipse.edc.spi.types.TypeManager;

@Extension(value = DataPlaneIonosS3Extension.NAME)
public class DataPlaneIonosS3Extension implements ServiceExtension {

    public static final String NAME = "Data Plane Ionos S3 Storage";
    @Inject
    private PipelineService pipelineService;
    
    @Inject
    private S3ConnectorApi s3Api;
    
    @Inject
    private DataTransferExecutorServiceContainer executorContainer;
    
    @Inject
    private Vault vault;

    @Inject
    private TypeManager typeManager;

    @Override
    public String name() {
        return NAME;
    }

    @Override
    public void initialize(ServiceExtensionContext context) {
        

        var monitor = context.getMonitor();
        
        var sourceFactory = new IonosDataSourceFactory(s3Api,typeManager);
        pipelineService.registerFactory(sourceFactory);
        
        var sinkFactory = new IonosDataSinkFactory(s3Api, executorContainer.getExecutorService(), monitor, vault,
            typeManager);
        pipelineService.registerFactory(sinkFactory);
        context.getMonitor().info("File Transfer Extension initialized!");
    }
}
