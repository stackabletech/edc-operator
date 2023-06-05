
# EDC Demo

This demo starts two operators (_provider_ and _consumer_) and executes a data transfer from one IONOS S3 bucket to another.

## Setup

What you need to do beforehand

- setup the buckets (two buckets, one with the demo file)

- get an ionos API token and S3 credentials (access key, secret key)
- a running kind cluster

run `make secret-manifests` and put in your ionos token and s3 credentials. This will create two files `ionos-token.yaml` and `s3-secret.yaml`.

## Running

First, run `make start`. This will:

- Install the secret and commons Operator into your kubernetes cluster
- Install a hashicorp Vault dev server for the consumer to use
- Install the CRD
- Install the manifests
- Run the Operator once to start the EDCs

Then, your cluster contains all the software needed to actually make the API calls and do the exchange.

Next you can run `make file-exchange` to do the actually exchange. That script will:

- Fetch provider and consumer endpoints via `kubectl`
- Execute the 7 API calls needed to define the asset, the policy, the contract, negotiate it and finally transfer the asset. Each step requires you to hit enter.