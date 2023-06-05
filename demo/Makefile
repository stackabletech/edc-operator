kind-load:
	kind load docker-image docker.stackable.tech/sandbox/edc:77c6e17-stackable0.0.0-dev

dependencies-logging:
	stackablectl stack install logging

dependencies:
	stackablectl operator install commons secret

consumer-vault:
	helm install \
    --set server.dev.enabled=true \
    --set server.dev.devRootToken=dev-token \
    consumer-vault hashicorp/vault

cert-secret:
	kubectl create secret generic connector-cert \
	--from-file=resources/cert.pfx \
	--from-file=resources/vault.properties

apply-crd:
	(cd .. && cargo run -- crd | kubectl apply -f -)

apply-secrets:
	kubectl apply -f s3-credentials-class.yaml
	kubectl apply -f s3-secret.yaml
	kubectl apply -f ionos-token.yaml

apply-demo:
	kubectl apply -f demo.yaml

apply-demo-logging:
	kubectl apply -f demo-logging.yaml

run:
	(cd .. && timeout --preserve-status 10s cargo run -- run)

secret-manifests:
	bash create-secrets.sh

start: dependencies consumer-vault cert-secret apply-crd apply-secrets apply-demo run

start-with-logging: dependencies-logging consumer-vault cert-secret apply-crd apply-secrets apply-demo-logging run

file-exchange:
	bash fileExchange.sh
