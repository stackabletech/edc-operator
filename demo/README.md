
# EDC Demo

This demo starts two operators (_provider_ and _consumer_) and executes a data transfer from one IONOS S3 bucket to another.

## Setup

What you need to do beforehand

- Set up two buckets in the IONOS DCD. One as a data source and one as a destination. Put the `device1-data.csv` file in the source bucket.
- Get an IONOS API token and IONOS S3 credentials (access key, secret key).
- Start a kind cluster.

Have the bucket names, the token and s3 credentials ready.
## Running

First, run `make start`. This will:

- Install the secret and commons Operator into your kubernetes cluster
- Install a hashicorp Vault dev server for the consumer to use
- Install the CRD
- Install the manifests
- Run the Operator once to start the EDCs

Along the way you will need to provide the information from the setup step. This will generate files, so you will not need to provide the information again when you start the demo again. To remove the generated files, call `make clean`.

Your cluster now contains all the software needed to actually make the API calls and do the exchange.

Next you can run `make file-exchange` to do the actually exchange. That script will:

- Fetch provider and consumer endpoints via `kubectl`
- Execute the 7 API calls needed to define the asset, the policy, the contract, negotiate it and finally transfer the asset. Each step requires you to hit enter.