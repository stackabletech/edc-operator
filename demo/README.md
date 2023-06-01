
# EDC Demo

This demo starts two operators (_provider_ and _consumer_) and executes a data transfer from one IONOS S3 bucket to another.

## Setup

What you need to do beforehand

- setup the buckets (two buckets, one with the demo file)
- get a token and put it into the demo.yaml where it says 'YOUR_TOKEN_HERE'
- get S3 credentials and create a secret called `s3-secret.yaml` using the `s3-secret-example.yaml` file
- configure all of that in the right places
- Start a kind cluster

## Running

First, run `make start`. This will:

- Install the secret and commons Operator into your kubernetes cluster
- Install a hashicorp Vault dev server for the consumer to use
- Install the CRD
- Install the manifests
- Run the Operator once to start the EDCs

Then, your cluster contains all the software needed to actually make the API calls and do the exchange.
Next you can run `bash fileExchange.sh` to do the actually exchange. That script will:

- Fetch provider and consumer endpoints via `kubectl`
- Execute the 7 API calls needed to define the asset, the policy, the contract, negotiate it and finally transfer the asset. Each step requires you to hit enter.