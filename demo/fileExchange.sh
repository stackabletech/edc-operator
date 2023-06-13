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

curl http://$PROVIDER_IP:$PROVIDER_PORT/api/v1/data/assets \
-H 'X-API-Key: password' \
-H 'content-type: application/json' \
-d @- <<-EOF
  {
    "asset": {
      "properties": {
        "asset:prop:id": "assetId",
        "asset:prop:name": "product description",
        "asset:prop:contenttype": "application/json"
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

curl http://$PROVIDER_IP:$PROVIDER_PORT/api/v1/data/policydefinitions \
-H 'X-API-Key: password' \
-H 'content-type: application/json' \
-d '{
  "id": "aPolicy",
  "policy": {
    "uid": "231802-bb34-11ec-8422-0242ac120002",
    "permissions": [
      {
        "target": "assetId",
        "action": {
          "type": "USE"
        },
        "edctype": "dataspaceconnector:permission"
      }
    ],
    "@type": {
      "@policytype": "set"
    }
  }
}'

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 3: Creating the contract in the provider:"

curl http://$PROVIDER_IP:$PROVIDER_PORT/api/v1/data/contractdefinitions \
-H 'X-API-Key: password' \
-H 'content-type: application/json' \
-d '{
   "id": "1",
   "accessPolicyId": "aPolicy",
   "contractPolicyId": "aPolicy",
   "criteria": []
 }'

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 4: Make the consumer fetch the data catalog from the provider:"

curl -X POST http://$CONSUMER_IP:$CONSUMER_PORT/api/v1/data/catalog/request \
--header 'X-API-Key: password' \
--header 'Content-Type: application/json' \
-d '{
  "providerUrl": "http://provider:8282/api/v1/ids/data"
}'

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 5: Contract Negotiation"

JSON_PAYLOAD=$(cat <<-EOF
{
  "connectorId": "multicloud-push-provider",
  "connectorAddress": "http://provider:8282/api/v1/ids/data",
  "protocol": "ids-multipart",
  "offer": {
    "offerId": "1:50f75a7a-5f81-4764-b2f9-ac258c3628e2",
    "assetId": "assetId",
    "policy": {
      "uid": "231802-bb34-11ec-8422-0242ac120002",
      "permissions": [
        {
        "target": "assetId",
        "action": {
            "type": "USE"
        },
        "edctype": "dataspaceconnector:permission"
        }
      ],
      "@type": {
        "@policytype": "set"
      }
    }
  }
}
EOF
)
ID=$(curl -s -X POST -H 'X-API-Key: password' -H 'content-type: application/json' -d "$JSON_PAYLOAD" "http://$CONSUMER_IP:$CONSUMER_PORT/api/v1/data/contractnegotiations" | jq -r '.id')
echo $ID

# This step takes a bit of time, sleep a bit
echo "..."
sleep 6

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 6: Fetching the contract agreement ID"

CONTRACT_AGREEMENT_ID=$(curl -s -X GET "http://$CONSUMER_IP:$CONSUMER_PORT/api/v1/data/contractnegotiations/$ID" \
	-H 'X-API-Key: password' \
  -H 'Content-Type: application/json' | jq -r '.contractAgreementId')
echo $CONTRACT_AGREEMENT_ID

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 7: Asset Transfer"

curl -X POST "http://$CONSUMER_IP:$CONSUMER_PORT/api/v1/data/transferprocess" \
-H "Content-Type: application/json" \
-H 'X-API-Key: password' \
-d @- <<-EOF
{
  "connectorId": "consumer",
  "connectorAddress": "http://provider:8282/api/v1/ids/data",
  "contractId": "$CONTRACT_AGREEMENT_ID",
  "protocol": "ids-multipart",
  "assetId": "assetId",
  "managedResources": "true",
  "transferType": {
    "contentType": "application/octet-stream",
    "isFinite": true
  },
  "dataDestination": {
    "properties": {
      "type": "IonosS3",
      "storage":"s3-eu-central-1.ionoscloud.com",
      "bucketName": "$DEST_BUCKET"
    }
  }
}
EOF

# Maybe echo something like: Transfer completed from <source> to <dest>
echo ""
