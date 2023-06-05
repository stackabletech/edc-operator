kind-load:
	kind load docker-image docker.stackable.tech/sandbox/edc:77c6e17-stackable0.0.0-dev

dependencies:
	stackablectl operator install commons secret

consumer-vault:
	helm install \
    --set server.dev.enabled=true \
    --set server.dev.devRootToken=dev-token \
    consumer-vault hashicorp/vault

apply-crd:
	(cd .. && cargo run -- crd | kubectl apply -f -)

apply-example:
	kubectl apply -f s3-credentials-class.yaml
	kubectl apply -f s3-secret.yaml
	kubectl apply -f ionos-token.yaml
	kubectl apply -f demo.yaml

cert-secret:
	kubectl create secret generic connector-cert \
	--from-file=resources/cert.pfx \
	--from-file=resources/vault.properties

apply: cert-secret apply-crd apply-example

run:
	(cd .. && timeout --preserve-status 10s cargo run -- run)

secret-manifests:
	bash create-secrets.sh

start: dependencies consumer-vault apply run

file-exchange:
	bash fileExchange.sh
