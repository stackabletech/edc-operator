# Set the DEMO_LOGGING environment variable to install logging


kind-load:
	kind load docker-image docker.stackable.tech/sandbox/edc:77c6e17-stackable0.0.0-dev

manifests/source-bucket.yaml:
	read -rp 'Enter your source bucket name: ' BUCKET && export BUCKET && \
	yq '.metadata.name = "source-bucket" | .spec.bucketName = env(BUCKET)' example-manifests/bucket.yaml > manifests/source-bucket.yaml

manifests/destination-bucket.yaml:
	read -rp 'Enter your destination bucket name: ' BUCKET && export BUCKET && \
	yq '.metadata.name = "destination-bucket" | .spec.bucketName = env(BUCKET)' example-manifests/bucket.yaml > manifests/destination-bucket.yaml

manifests/ionos-token.yaml:
	read -rp 'Enter your IONOS API token: ' TOKEN && export TOKEN && \
	yq '.stringData.EDC_IONOS_TOKEN =  env(TOKEN)' example-manifests/ionos-token-secret.yaml > manifests/ionos-token.yaml

manifests/s3-secret.yaml:
	read -rp 'Enter your IONOS S3 access key: ' ACCESS_KEY && export ACCESS_KEY && \
	read -rp 'Enter your IONOS S3 secret key: ' SECRET_KEY && export SECRET_KEY && \
	yq '.stringData.accessKey = env(ACCESS_KEY) | .stringData.secretKey = env(SECRET_KEY)' example-manifests/s3-secret.yaml > manifests/s3-secret.yaml

dependencies:
	if [ -z "$(DEMO_LOGGING)" ]; \
	then stackablectl operator install commons=23.4.1 secret=23.4.1 ; \
	else stackablectl stack install logging; fi

consumer-vault:
	echo "Installing Hashicorp Vault"
	helm install \
	--repo https://helm.releases.hashicorp.com \
    --set server.dev.enabled=true \
    --set server.dev.devRootToken=dev-token \
    --wait \
    consumer-vault vault

install-operator:
	helm install --wait --devel --repo https://repo.stackable.tech/repository/helm-dev/ edc-operator edc-operator

cert-secret:
	kubectl delete secret connector-cert --ignore-not-found
	kubectl create secret generic connector-cert \
	--from-file=resources/cert.pfx \
	--from-file=resources/vault.properties

apply: cert-secret manifests/source-bucket.yaml manifests/destination-bucket.yaml manifests/s3-secret.yaml manifests/ionos-token.yaml
	kubectl apply -f manifests/s3-credentials-class.yaml
	kubectl apply -f manifests/s3-secret.yaml
	kubectl apply -f manifests/ionos-token.yaml
	kubectl apply -f manifests/source-bucket.yaml
	kubectl apply -f manifests/destination-bucket.yaml
	if [ -z "$(DEMO_LOGGING)" ]; then kubectl apply -f manifests/demo.yaml; else kubectl apply -f manifests/demo-logging.yaml; fi

### FOR DEVELOPMENT

apply-crd:
	(cd .. && cargo run -- crd | kubectl apply -f -)

run:
	(cd .. && timeout --preserve-status 10s cargo run -- run)

### MAIN STUFF

start-dev: dependencies consumer-vault apply-crd apply run

start: dependencies consumer-vault install-operator apply

# The file exchange needs these two files, because the bucket names are read from them
file-exchange: manifests/source-bucket.yaml manifests/destination-bucket.yaml
	bash fileExchange.sh

# removes all generated files
clean:
	rm manifests/ionos-token.yaml
	rm manifests/s3-secret.yaml
	rm manifests/destination-bucket.yaml
	rm manifests/source-bucket.yaml
