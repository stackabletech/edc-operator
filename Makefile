kind-load:
	kind load docker-image docker.stackable.tech/sandbox/edc:0.0.1-SNAPSHOT-stackable0.0.0-dev

dependencies:
	stackablectl op in commons secret

apply-crd:
	cargo run -- crd | kubectl apply -f -

apply-example:
	kubectl apply -f manifests/s3-credentials-class.yaml
	kubectl apply -f manifests/s3-secret.yaml
	kubectl apply -f manifests/edc.yaml

cert-secret:
	kubectl create secret generic connector-cert \
	--from-file=resources/cert.pfx \
	--from-file=resources/vault.properties

apply: cert-secret apply-crd apply-example

run:
	cargo run -- run

start: dependencies apply run

pf:
	kubectl port-forward svc/connector 8181 &
	kubectl port-forward svc/connector 8182 &
	kubectl port-forward svc/connector 8282 &

kill-pf:
	ps aux | grep '[k]ubectl port-forward' | awk -F ' ' '{print $$2}' | xargs kill

