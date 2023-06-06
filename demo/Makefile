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

dependencies-logging:
	stackablectl stack install logging

dependencies:
	stackablectl operator install commons secret

consumer-vault:
	helm install \
	--repo https://helm.releases.hashicorp.com \
    --set server.dev.enabled=true \
    --set server.dev.devRootToken=dev-token \
    consumer-vault vault

cert-secret:
	kubectl delete secret connector-cert --ignore-not-found
	kubectl create secret generic connector-cert \
	--from-file=resources/cert.pfx \
	--from-file=resources/vault.properties

apply-crd:
	(cd .. && cargo run -- crd | kubectl apply -f -)

apply-deps: cert-secret manifests/source-bucket.yaml manifests/destination-bucket.yaml manifests/s3-secret.yaml manifests/ionos-token.yaml
	kubectl apply -f manifests/s3-credentials-class.yaml
	kubectl apply -f manifests/s3-secret.yaml
	kubectl apply -f manifests/ionos-token.yaml
	kubectl apply -f manifests/source-bucket.yaml
	kubectl apply -f manifests/destination-bucket.yaml

apply-demo: apply-crd apply-deps
	kubectl apply -f manifests/demo.yaml

apply-demo-logging: apply-crd apply-deps
	kubectl apply -f manifests/demo-logging.yaml

run:
	(cd .. && timeout --preserve-status 10s cargo run -- run)

start: dependencies consumer-vault apply-demo run

start-with-logging: dependencies-logging consumer-vault apply-demo-logging run

# The file exchange needs these two files, because the bucket names are read from them
file-exchange: manifests/source-bucket.yaml manifests/destination-bucket.yaml
	bash fileExchange.sh

clean:
	rm manifests/ionos-token.yaml
	rm manifests/s3-secret.yaml
	rm manifests/destination-bucket.yaml
	rm manifests/source-bucket.yaml