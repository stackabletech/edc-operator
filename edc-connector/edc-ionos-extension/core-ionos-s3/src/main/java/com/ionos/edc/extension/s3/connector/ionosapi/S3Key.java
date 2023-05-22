package com.ionos.edc.extension.s3.connector.ionosapi;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Map;

public class S3Key {
    private String id;
    private String type;
    private String href;
    private Metadata metadata;
    private Map<String, Object> properties;
	public String getId() {
		return id;
	}
	public void setId(String id) {
		this.id = id;
	}
	public String getType() {
		return type;
	}
	public void setType(String type) {
		this.type = type;
	}
	public String getHref() {
		return href;
	}
	public void setHref(String href) {
		this.href = href;
	}
	public Metadata getMetadata() {
		return metadata;
	}
	public void setMetadata(Metadata metadata) {
		this.metadata = metadata;
	}
	public Map<String, Object> getProperties() {
		return properties;
	}
	public void setProperties(Map<String, Object> properties) {
		this.properties = properties;
	}

  
}

class Metadata {
    private String etag;
    @JsonProperty("createdDate")
    private String createdDate;
	public String getEtag() {
		return etag;
	}
	public void setEtag(String etag) {
		this.etag = etag;
	}
	public String getCreatedDate() {
		return createdDate;
	}
	public void setCreatedDate(String createdDate) {
		this.createdDate = createdDate;
	}


}