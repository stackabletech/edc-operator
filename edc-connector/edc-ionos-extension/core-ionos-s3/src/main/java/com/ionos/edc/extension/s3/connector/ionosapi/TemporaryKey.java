package com.ionos.edc.extension.s3.connector.ionosapi;

public class TemporaryKey {

	public String accessKey;
	public String secretKey;
	
	
	
	public TemporaryKey(String accessKey, String secretKey) {
		super();
		this.accessKey = accessKey;
		this.secretKey = secretKey;
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
	
	
}
