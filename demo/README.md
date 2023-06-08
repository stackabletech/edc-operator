
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

### Logging

If the `DEMO_LOGGING` variable is set, the logging Stack will also be installed (i.e. run `DEMO_LOGGING=true make start`). You can use `stackablectl svc list` to see how to access to logging UI.

## More info

You can install the Operator with 

    helm install edc-operator stackable-dev/edc-operator --devel

The `stackable-dev` repo is at `https://repo.stackable.tech/repository/helm-dev/`. (Use `helm repo add stackable-dev https://repo.stackable.tech/repository/helm-dev/` to add it).

In the `manifests` directory you will find all the files that get deployed:

- `demo.yaml` (or `demo-logging.yaml`) contain the main EDCCluster definitions. They reference the other files.
- `ionos-token.yaml` is a simple Secret that contains the IONOS API token. It is referenced in the `demo.yaml`.
- `s3-credentials-class.yaml` and `s3-secret.yaml` define the SecretClass and credentials secret respectively. Have a look at the [docs](https://docs.stackable.tech/home/stable/concepts/s3.html#_credentials) to read more about S3 credentials for Stackable Operators.
- `source-bucket.yaml` and `destination-bucket.yaml` are also referenced in the `demo.yaml` file. They both reference the S3 credentials SecretClass.


### Custom configuration and overrides

In addition to the default configuration settings passed to the EDC, you can supply your own. You can also override settings that the Operato sets. An example:

```
spec:
  ...
  connectors:
    roleGroups:
      default:
        replicas: 1
        configOverrides:
          config.properties:
            edc.vault.name: my-vault
```

Here the `edc.vault.name` settings is overwritten. This setting will be added to the config file in the ConfigMap that gets mounted into the EDC container.
