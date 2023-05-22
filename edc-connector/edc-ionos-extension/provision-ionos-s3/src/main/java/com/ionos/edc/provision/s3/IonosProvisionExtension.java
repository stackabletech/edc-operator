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

package com.ionos.edc.provision.s3;

import com.ionos.edc.extension.s3.api.S3ConnectorApi;
import com.ionos.edc.extension.s3.configuration.IonosToken;
import com.ionos.edc.extension.s3.schema.IonosBucketSchema;
import com.ionos.edc.provision.s3.bucket.IonosS3ConsumerResourceDefinitionGenerator;
import com.ionos.edc.provision.s3.bucket.IonosS3ProvisionedResource;
import com.ionos.edc.provision.s3.bucket.IonosS3Provisioner;
import com.ionos.edc.provision.s3.bucket.IonosS3ResourceDefinition;
import com.ionos.edc.provision.s3.bucket.IonosS3StatusChecker;
import dev.failsafe.RetryPolicy;
import org.eclipse.edc.connector.transfer.spi.provision.ProvisionManager;
import org.eclipse.edc.connector.transfer.spi.provision.ResourceManifestGenerator;
import org.eclipse.edc.connector.transfer.spi.status.StatusCheckerRegistry;
import org.eclipse.edc.runtime.metamodel.annotation.Extension;
import org.eclipse.edc.runtime.metamodel.annotation.Inject;
import org.eclipse.edc.spi.monitor.Monitor;
import org.eclipse.edc.spi.security.Vault;
import org.eclipse.edc.spi.system.ServiceExtension;
import org.eclipse.edc.spi.system.ServiceExtensionContext;
import org.eclipse.edc.spi.types.TypeManager;




@Extension(value = IonosProvisionExtension.NAME)
public class IonosProvisionExtension implements ServiceExtension {

    public static final String NAME = "Ionos Provision";

    @Inject
    private Vault vault;
    @Inject
    private Monitor monitor;

    @Inject
    S3ConnectorApi clientApi;

    @Override
    public String name() {
        return NAME;
    }

    @Override
    public void initialize(ServiceExtensionContext context) {
        // TODO Auto-generated method stub
        monitor = context.getMonitor();
        monitor.debug("IonosProvisionExtension" + "provisionManager");
        var provisionManager = context.getService(ProvisionManager.class);
        monitor.debug("IonosProvisionExtension" + "retryPolicy");
        var retryPolicy = (RetryPolicy<Object>) context.getService(RetryPolicy.class);
        monitor.debug("IonosProvisionExtension" + "s3BucketProvisioner");
        var s3BucketProvisioner = new IonosS3Provisioner(retryPolicy, monitor, clientApi);
        provisionManager.register(s3BucketProvisioner);

        // register the generator
        monitor.debug("IonosProvisionExtension" + "manifestGenerator");
        var manifestGenerator = context.getService(ResourceManifestGenerator.class);
        manifestGenerator.registerGenerator(new IonosS3ConsumerResourceDefinitionGenerator());
        monitor.debug("IonosProvisionExtension" + "statusCheckerReg");
        var statusCheckerReg = context.getService(StatusCheckerRegistry.class);
        statusCheckerReg.register(IonosBucketSchema.TYPE, new IonosS3StatusChecker(clientApi, retryPolicy));
        monitor.debug("IonosProvisionExtension" + "registerTypes");
        registerTypes(context.getTypeManager());
    }

    @Override
    public void shutdown() {
        ServiceExtension.super.shutdown();
    }

    private void registerTypes(TypeManager typeManager) {
        typeManager.registerTypes(IonosS3ProvisionedResource.class, IonosS3ResourceDefinition.class, IonosToken.class);
    }
}
