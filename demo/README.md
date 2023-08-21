
# EDC Demo

In this demo, the EDC operator is used to set up two EDC instances (_provider_ and _consumer_) with IONOS S3 buckets as storage. Then a file transfer between the two connectors is performed. The demo is based on the [file-transfer-multiple-instances example](https://github.com/Digital-Ecosystems/edc-ionos-s3/tree/main/example/file-transfer-multiple-instances) by IONOS, which in turn is based on the [transfer-06-consumer-pull-http example](https://github.com/eclipse-edc/Samples/tree/main/transfer/transfer-06-consumer-pull-http) in the upstream EDC repository.

The data exchange is facilitated by the [Dataspace Protocol](https://docs.internationaldataspaces.org/ids-knowledgebase/v/dataspace-protocol/overview/readme). The linked document describes the individual steps, the messages, properties and state machines involved.

## Setup

What you need to do beforehand

- Access to a Kubernetes cluster. We will use `kind` for this demo.
- The `stackablectl` and `helm` command line tools for installing the Stackable Platform Oerators.
- The `ionosctl` command line tool and an IONOS DCD account.
- Two S3 buckets in the IONOS DCD. One as a data source and one as a destination. Put the `device1-data.csv` file in the source bucket.
- An IONOS API token. Use the `ionosctl token generate` to generate one.
- IONOS S3 credentials (access key, secret key). Use `ionosctl user s3key get --user-id <your-user-id> --s3key-id <key-id> -o json` to obtain the secret key of existing S3 credentials.
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
