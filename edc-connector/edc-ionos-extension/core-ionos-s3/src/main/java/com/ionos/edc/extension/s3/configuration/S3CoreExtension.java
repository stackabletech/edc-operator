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

import com.ionos.edc.extension.s3.api.S3ConnectorApi;
import com.ionos.edc.extension.s3.api.S3ConnectorApiImpl;
import org.eclipse.edc.runtime.metamodel.annotation.Extension;
import org.eclipse.edc.runtime.metamodel.annotation.Inject;
import org.eclipse.edc.runtime.metamodel.annotation.Provides;
import org.eclipse.edc.runtime.metamodel.annotation.Setting;
import org.eclipse.edc.spi.monitor.Monitor;
import org.eclipse.edc.spi.security.Vault;
import org.eclipse.edc.spi.system.ServiceExtension;
import org.eclipse.edc.spi.system.ServiceExtensionContext;
import org.eclipse.edc.spi.security.CertificateResolver;
import org.eclipse.edc.spi.security.PrivateKeyResolver;
import org.eclipse.edc.spi.system.vault.NoopCertificateResolver;
import org.eclipse.edc.spi.system.vault.NoopPrivateKeyResolver;

@Provides(S3ConnectorApi.class)
@Extension(value = S3CoreExtension.NAME)
public class S3CoreExtension implements ServiceExtension {
    public static final String NAME = "IonosS3";
    @Setting
    private static final String IONOS_ACCESS_KEY = "edc.ionos.access.key";
    @Setting
    private static final String IONOS_SECRET_KEY = "edc.ionos.secret.key";
    @Setting
    private static final String IONOS_ENDPOINT = "edc.ionos.endpoint";
    @Setting
    private static final String IONOS_TOKEN = "edc.ionos.token";
    @Setting
    private static final String IONOS_WITH_VAULT = "edc.ionos.vault";
    
    @Inject
    private Vault vault;

    @Inject
    private Monitor monitor;

    @Override
    public String name() {
        return NAME;
    }

    @Override
    public void initialize(ServiceExtensionContext context) {
        var accessKey = vault.resolveSecret(IONOS_ACCESS_KEY);
        var secretKey = vault.resolveSecret(IONOS_SECRET_KEY);
        var endPoint = vault.resolveSecret(IONOS_ENDPOINT);
        var token =  vault.resolveSecret(IONOS_TOKEN);
        if(accessKey == null || secretKey  == null || endPoint ==null) {    	
        	  accessKey = context.getSetting(IONOS_ACCESS_KEY, IONOS_ACCESS_KEY);
              secretKey = context.getSetting(IONOS_SECRET_KEY, IONOS_SECRET_KEY);
              endPoint = context.getSetting(IONOS_ENDPOINT, IONOS_ENDPOINT);
              token = context.getSetting(IONOS_TOKEN, IONOS_TOKEN);
        }
		
        var s3Api = new S3ConnectorApiImpl(endPoint, accessKey, secretKey, token);
        context.registerService(S3ConnectorApi.class, s3Api);

        // var privateKeyResolver = new NoopPrivateKeyResolver();
        // context.registerService(PrivateKeyResolver.class, privateKeyResolver);

        // var certificateResolver = new NoopCertificateResolver();
        // context.registerService(CertificateResolver.class, certificateResolver);
    }

}
