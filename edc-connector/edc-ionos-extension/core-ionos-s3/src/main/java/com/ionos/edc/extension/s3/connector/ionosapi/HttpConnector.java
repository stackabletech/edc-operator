package com.ionos.edc.extension.s3.connector.ionosapi;

import java.io.IOException;

import com.fasterxml.jackson.databind.ObjectMapper;

import okhttp3.FormBody;
import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.RequestBody;
import okhttp3.Response;


public class HttpConnector {
	OkHttpClient client = new OkHttpClient();
	String basicUrl = "https://api.ionos.com/cloudapi/v6/um/users/";
	
	public String retrieveUserID(String token)  {		
		String[] jwtParts = token.split("\\.");
		String jwtPayload = new String(java.util.Base64.getDecoder().decode(jwtParts[1]));
		String uuid = jwtPayload.split("\"uuid\":\"")[1].split("\"")[0];

		return uuid;
	}
	
		
	public TemporaryKey createTemporaryKey(String token) {
		 String url = basicUrl + retrieveUserID(token) + "/s3keys";
		
		
		Request request = new Request.Builder()
				   .url(url)
				   //This adds the token to the header.
				   .addHeader("Authorization", "Bearer " + token)
				   .post(RequestBody.create(null, new byte[0]))
				   .build();
				    try (Response response = client.newCall(request).execute()) {
				         if (!response.isSuccessful()){
				            throw new IOException("Unexpected code " + response);
				         }


					        ObjectMapper objectMapper = new ObjectMapper();
					        S3Key resp = objectMapper.readValue(response.body().string(), S3Key.class);					        
					        TemporaryKey temp = new TemporaryKey(resp.getId().toString(),resp.getProperties().get("secretKey").toString());
					        return temp;
				    } catch (IOException e) {
						e.printStackTrace();
						return new TemporaryKey("", "");
					}				    
	}
	
	public void deleteTemporaryAccount(String token, String keyID)  {
		String url = basicUrl + retrieveUserID(token) + "/s3keys/" + keyID;

		Request request = new Request.Builder()
				   .url(url)
				   //This adds the token to the header.
				   .addHeader("Authorization", "Bearer " + token)
				   .delete()
				   .build();
				    try (Response response = client.newCall(request).execute()) {
				         if (!response.isSuccessful()){
				            throw new IOException("Unexpected code " + response);
				         }
				     } catch (IOException e) {				
						e.printStackTrace();
					}
	}
	
}
