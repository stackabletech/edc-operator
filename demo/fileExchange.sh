#!/bin/bash
set -euo pipefail

PROVIDER_IP=$(kubectl get pod provider-server-default-0 --output=jsonpath='{.status.hostIP}')
PROVIDER_PORT=$(kubectl get svc provider --output=jsonpath='{.spec.ports[?(@.name=="management")].nodePort}')
CONSUMER_IP=$(kubectl get pod consumer-server-default-0 --output=jsonpath='{.status.hostIP}')
CONSUMER_PORT=$(kubectl get svc consumer --output=jsonpath='{.spec.ports[?(@.name=="management")].nodePort}')

# TODO use something like this to make the bucket configurable
#  echo null | jq '{
#     asset: {},
#     dataAddress: {
#         properties: {
#             bucketName: $ENV.BUCKET_NAME,
#         },
#     },
# }' | whereveritneedstogo
#
# for curl you should be able to use --json @- to load the data from stdin


echo "################################################"
echo "Step 1: Creating the asset in the provider:"

curl --header 'X-API-Key: password' \
-d '{
           "asset": {
             "properties": {
               "asset:prop:id": "assetId",
               "asset:prop:name": "product description",
               "asset:prop:contenttype": "application/json"
             }
           },
           "dataAddress": {
             "properties": {
			   "bucketName": "source-bucket",
			   "container": "source-bucket",
               "blobName": "device1-data.csv",
               "storage": "s3-eu-central-1.ionoscloud.com",
               "keyName": "device1-data.csv",
               "type": "IonosS3"
             }
           }
         }' -H 'content-type: application/json' http://$PROVIDER_IP:$PROVIDER_PORT/api/v1/data/assets \
         -s

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 2: Creating the policy in the provider:"

curl -d '{
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
         }' -H 'X-API-Key: password' \
		 -H 'content-type: application/json' http://$PROVIDER_IP:$PROVIDER_PORT/api/v1/data/policydefinitions		

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 3: Creating the contract in the provider:"

curl -d '{
   "id": "1",
   "accessPolicyId": "aPolicy",
   "contractPolicyId": "aPolicy",
   "criteria": []
 }' -H 'X-API-Key: password' \
 -H 'content-type: application/json' http://$PROVIDER_IP:$PROVIDER_PORT/api/v1/data/contractdefinitions

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 4: Make the consumer fetch the data catalog from the provider:"

curl -X POST http://$CONSUMER_IP:$CONSUMER_PORT/api/v1/data/catalog/request \
--header 'X-API-Key: password' \
--header 'Content-Type: application/json' \
-d @- <<-EOF
{
  "providerUrl": "http://provider:8282/api/v1/ids/data"
}
EOF

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
ID=$(curl -s --header 'X-API-Key: password' -X POST -H 'content-type: application/json' -d "$JSON_PAYLOAD" "http://$CONSUMER_IP:$CONSUMER_PORT/api/v1/data/contractnegotiations" | jq -r '.id')
echo $ID

# This step takes a bit of time, sleep a bit
echo "..."
sleep 6

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 6: Fetching the contract agreement ID"

CONTRACT_AGREEMENT_ID=$(curl -X GET "http://$CONSUMER_IP:$CONSUMER_PORT/api/v1/data/contractnegotiations/$ID" \
	--header 'X-API-Key: password' \
    --header 'Content-Type: application/json' \
    -s | jq -r '.contractAgreementId')
echo $CONTRACT_AGREEMENT_ID

read -p $'\n\nPress enter to continue'

echo "################################################"
echo "Step 7: Asset Transfer"

curl -X POST "http://$CONSUMER_IP:$CONSUMER_PORT/api/v1/data/transferprocess" \
    --header "Content-Type: application/json" \
	--header 'X-API-Key: password' \
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
                "bucketName": "destination-bucket"
            }
        }
    }
EOF

