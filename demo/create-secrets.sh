#!/bin/bash

read -rp 'Enter your IONOS TOKEN: ' EDC_IONOS_TOKEN

export EDC_IONOS_TOKEN

cat ionos-token-example.yaml | yq '.stringData.EDC_IONOS_TOKEN = env(EDC_IONOS_TOKEN)' > ionos-token.yaml

echo 'ionos-token.yaml written'

read -rp 'Enter your IONOS S3 access key: ' ACCESS_KEY
read -rp 'Enter your IONOS S3 secret key: ' SECRET_KEY

export ACCESS_KEY SECRET_KEY

cat s3-secret-example.yaml | yq '.stringData.accessKey = env(ACCESS_KEY)' | yq '.stringData.secretKey = env(SECRET_KEY)'  > s3-secret.yaml

echo 's3-secret-yaml written'
