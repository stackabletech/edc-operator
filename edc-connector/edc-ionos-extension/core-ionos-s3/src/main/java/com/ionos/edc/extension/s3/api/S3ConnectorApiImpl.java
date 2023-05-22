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

package com.ionos.edc.extension.s3.api;

import com.ionos.edc.extension.s3.connector.MinioConnector;
import com.ionos.edc.extension.s3.connector.ionosapi.HttpConnector;
import com.ionos.edc.extension.s3.connector.ionosapi.TemporaryKey;

import io.minio.BucketExistsArgs;
import io.minio.GetObjectArgs;
import io.minio.ListObjectsArgs;
import io.minio.MakeBucketArgs;
import io.minio.MinioClient;
import io.minio.PutObjectArgs;
import io.minio.RemoveBucketArgs;
import io.minio.RemoveObjectArgs;
import io.minio.Result;
import io.minio.UploadObjectArgs;
import io.minio.errors.ErrorResponseException;
import io.minio.errors.InsufficientDataException;
import io.minio.errors.InternalException;
import io.minio.errors.InvalidResponseException;
import io.minio.errors.ServerException;
import io.minio.errors.XmlParserException;
import io.minio.messages.Item;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.security.InvalidKeyException;
import java.security.NoSuchAlgorithmException;


public class S3ConnectorApiImpl implements S3ConnectorApi {

    MinioConnector minConnector = new MinioConnector();
    HttpConnector ionosApi = new HttpConnector();
    
    private MinioClient minioClient;
    private String token;

    
    public S3ConnectorApiImpl(String endpoint, String accessKey, String secretKey, String token) {
    	if(accessKey != null && secretKey  != null && endpoint !=null)
    		this.minioClient = minConnector.connect(endpoint, accessKey, secretKey);
        this.token = token; 
    }

    @Override
    public void s3ConnectorApi(String endpoint, String accessKey, String secretKey, String token) {
    	if(accessKey != null && secretKey  != null && endpoint !=null)
    		this.minioClient = minConnector.connect(endpoint, accessKey, secretKey);
        this.token = token; 
    }

    @Override
    public void createBucket(String bucketName) {

        if (!bucketExists(bucketName.toLowerCase())) {
            // Make a new bucket'.
            try {
                minioClient.makeBucket(MakeBucketArgs.builder().bucket(bucketName.toLowerCase()).build());
            } catch (InvalidKeyException | ErrorResponseException | InsufficientDataException | InternalException |
                    InvalidResponseException | NoSuchAlgorithmException | ServerException | XmlParserException |
                    IllegalArgumentException | IOException e) {
                e.printStackTrace();
            }
        }

    }

    @Override
    public void deleteBucket(String bucketName) {

        if (bucketExists(bucketName.toLowerCase())) {
            // Remove a bucket.
            try {
                minioClient.removeBucket(RemoveBucketArgs.builder().bucket(bucketName.toLowerCase()).build());
            } catch (InvalidKeyException | ErrorResponseException | InsufficientDataException | InternalException |
                    InvalidResponseException | NoSuchAlgorithmException | ServerException | XmlParserException |
                    IllegalArgumentException | IOException e) {

                e.printStackTrace();
            }
        }
    }

    @Override
    public void uploadFile(String bucketName, String fileName, String path) {

        if (bucketExists(bucketName.toLowerCase())) {

            try {
                minioClient.uploadObject(UploadObjectArgs.builder().bucket(bucketName.toLowerCase()).object(fileName)
                        .filename(path).build());
            } catch (InvalidKeyException | ErrorResponseException | InsufficientDataException | InternalException |
                    InvalidResponseException | NoSuchAlgorithmException | ServerException | XmlParserException |
                    IllegalArgumentException | IOException e) {

                e.printStackTrace();
            }
        }
    }
    
    @Override
    public void uploadParts(String bucketName, String fileName, ByteArrayInputStream  part) {
        if (bucketExists(bucketName.toLowerCase())) {

            try {
                minioClient.putObject(PutObjectArgs.builder().bucket(bucketName).object(fileName).stream(part, part.available(), -1).build());
            } catch (InvalidKeyException | ErrorResponseException | InsufficientDataException | InternalException |
                    InvalidResponseException | NoSuchAlgorithmException | ServerException | XmlParserException |
                    IllegalArgumentException | IOException e) {

                e.printStackTrace();
            }
        }
    }

    @Override
    public byte[] getFile(String bucketName, String fileName) {

        if (!bucketExists(bucketName.toLowerCase())) {
            return null;
        }
        InputStream stream;
        try {
            stream = minioClient.getObject(GetObjectArgs.builder().bucket(bucketName).object(fileName).build());
            return stream.readAllBytes();
        } catch (InvalidKeyException | ErrorResponseException | InsufficientDataException | InternalException |
                InvalidResponseException | NoSuchAlgorithmException | ServerException | XmlParserException |
                IllegalArgumentException | IOException e) {

            e.printStackTrace();
            return null;
        }

    }

    @Override
    public Result<Item> listItems(String bucketName) {

        Iterable<Result<Item>> results = minioClient
                .listObjects(ListObjectsArgs.builder().bucket(bucketName.toLowerCase()).build());

        return results.iterator().next();
    }

    @Override
    public void deleteFile(String bucketName, String fileName) {

        try {
            minioClient.removeObject(RemoveObjectArgs.builder().bucket(bucketName.toLowerCase()).object(fileName).build());
        } catch (InvalidKeyException | ErrorResponseException | InsufficientDataException | InternalException |
                InvalidResponseException | NoSuchAlgorithmException | ServerException | XmlParserException |
                IllegalArgumentException | IOException e) {

            e.printStackTrace();
        }
    }

    @Override
    public boolean bucketExists(String bucketName) {

        boolean found = false;
        try {
        	
            found = minioClient.bucketExists(BucketExistsArgs.builder().bucket(bucketName.toLowerCase()).build());
        } catch (InvalidKeyException | ErrorResponseException | InsufficientDataException | InternalException |
                InvalidResponseException | NoSuchAlgorithmException | ServerException | XmlParserException |
                IllegalArgumentException | IOException e) {

            e.printStackTrace();
        }
        return found;
    }
    
    @Override
    public  TemporaryKey createTemporaryKey() {
    	
		return ionosApi.createTemporaryKey(token);
    	
    }
 
	@Override
	public void deleteTemporaryKey(String accessKey) {
		
		ionosApi.deleteTemporaryAccount(token,accessKey);
		
	}
    
}
