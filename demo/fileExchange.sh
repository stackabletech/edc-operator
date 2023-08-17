#!/bin/bash
set -euo pipefail

PROVIDER_IP=$(kubectl get pod provider-server-default-0 --output=jsonpath='{.status.hostIP}')
PROVIDER_PORT=$(kubectl get svc provider --output=jsonpath='{.spec.ports[?(@.name=="management")].nodePort}')
CONSUMER_IP=$(kubectl get pod consumer-server-default-0 --output=jsonpath='{.status.hostIP}')
CONSUMER_PORT=$(kubectl get svc consumer --output=jsonpath='{.spec.ports[?(@.name=="management")].nodePort}')

SRC_BUCKET="$(yq .spec.bucketName manifests/source-bucket.yaml)"
DEST_BUCKET="$(yq .spec.bucketName manifests/destination-bucket.yaml)"

echo "################################################"
echo "Consumer: $CONSUMER_IP:$CONSUMER_PORT"
echo "Provider: $PROVIDER_IP:$PROVIDER_PORT"
echo "Source bucket:      $SRC_BUCKET"
echo "Destination bucket: $DEST_BUCKET"
echo "################################################"

echo "################################################"
echo "Step 1: Creating the asset in the provider:"

curl http://$PROVIDER_IP:$PROVIDER_PORT/management/v2/assets \
-H 'X-API-Key: password' \
-H 'content-type: application/json' \
-d @- <<-EOF
  {
    "@context": {
      "edc": "https://w3id.org/edc/v0.0.1/ns/"
    },
    "asset": {
      "@id": "assetId",
      "properties": {    
        "name": "product description",
        "contenttype": "application/json"
      }
    },
    "dataAddress": {
      "properties": {
      "bucketName": "$SRC_BUCKET",
      "container": "$SRC_BUCKET",
        "blobName": "device1-data.csv",
        "storage": "s3-eu-central-1.ionoscloud.com",
        "keyName": "device1-data.csv",
        "type": "IonosS3"
      }
    }
  }
EOF

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 2: Creating the policy in the provider:"

curl http://$PROVIDER_IP:$PROVIDER_PORT/management/v2/policydefinitions \
-H 'X-API-Key: password' \
-H 'content-type: application/json' \
-d '{
      "@context": {
				"edc": "https://w3id.org/edc/v0.0.1/ns/",
				"odrl": "http://www.w3.org/ns/odrl/2/"
			},
      "@id": "aPolicy",
      "policy": {
        "@type": "set",
          "odrl:permission": [],
          "odrl:prohibition": [],
          "odrl:obligation": []
      }
    }'

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 3: Creating the contract in the provider:"

curl http://$PROVIDER_IP:$PROVIDER_PORT/management/v2/contractdefinitions \
-H 'X-API-Key: password' \
-H 'content-type: application/json' \
-d '{
      "@context": {
        "edc": "https://w3id.org/edc/v0.0.1/ns/"
      },
      "id": "1",
      "accessPolicyId": "aPolicy",
      "contractPolicyId": "aPolicy",
      "assetsSelector": []
    }'

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 4: Make the consumer fetch the data catalog from the provider:"

curl -X POST http://$CONSUMER_IP:$CONSUMER_PORT/management/v2/catalog/request \
--header 'X-API-Key: password' \
--header 'Content-Type: application/json' \
-d '{
      "@context": {
        "edc": "https://w3id.org/edc/v0.0.1/ns/"
      },
      "providerUrl": "http://PROVIDER_IP:8282/protocol",
      "protocol": "dataspace-protocol-http"
    }'

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 5: Contract Negotiation"

JSON_PAYLOAD=$(cat <<-EOF
{
  "@context": {
    "edc": "https://w3id.org/edc/v0.0.1/ns/",
    "odrl": "http://www.w3.org/ns/odrl/2/"
  },
  "@type": "NegotiationInitiateRequestDto",
  "connectorId": "provider",
  "connectorAddress": "http://provider:8282/protocol",
  "protocol": "dataspace-protocol-http",
  "offer": {
    "offerId": "1:1:a345ad85-c240-4195-b954-13841a6331a1",
    "assetId": "1",
    "policy": {"@id":"$OFFER_POLICY",
      "@type": "odrl:Set",
      "odrl:permission": {
        "odrl:target": "1",
        "odrl:action": {
          "odrl:type": "USE"
        }
      },
      "odrl:prohibition": [],
      "odrl:obligation": [],
      "odrl:target": "1"}
  }
}
EOF
)
ID=$(curl -s -X POST -H 'X-API-Key: password' -H 'content-type: application/json' -d "$JSON_PAYLOAD" "http://$CONSUMER_IP:$CONSUMER_PORT/management/v2/contractnegotiations" | jq -r '.["@id"]')
echo $ID

# This step takes a bit of time, sleep a bit
echo "..."
sleep 6

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 6: Fetching the contract agreement ID"

CONTRACT_AGREEMENT_ID=$(curl -s -X GET "http://$CONSUMER_IP:$CONSUMER_PORT/management/v2/contractnegotiations/$ID" \
	-H 'X-API-Key: password' \
  -H 'Content-Type: application/json' | jq -r '.["edc:contractAgreementId"]')
echo $CONTRACT_AGREEMENT_ID

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 7: Asset Transfer"

curl -X POST "http://$CONSUMER_IP:$CONSUMER_PORT/management/v2/transferprocess" \
-H "Content-Type: application/json" \
-H 'X-API-Key: password' \
-d @- <<-EOF
{	
  "@context": {
    "edc": "https://w3id.org/edc/v0.0.1/ns/"
    },
  "@type": "TransferRequestDto",
  "connectorId": "consumer",
  "connectorAddress": "http://provider:8282/protocol",
  "protocol": "dataspace-protocol-http",
  "contractId": "$CONTRACT_AGREEMENT_ID",
  "protocol": "ids-multipart",
  "assetId": "assetId",
  "dataDestination": { 
    "type": "IonosS3",
    "storage":"s3-eu-central-1.ionoscloud.com",
    "bucketName": "$CONSUMER_BUCKET",
    "keyName" : "device1-data.csv"
  
  },
  "managedResources": false
}
EOF

# Maybe echo something like: Transfer completed from <source> to <dest>
echo ""
