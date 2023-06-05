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
	kubectl apply -f manifests/vector-aggregator-discovery.yaml

delete-example:
	kubectl delete -f manifests/s3-credentials-class.yaml
	kubectl delete -f manifests/s3-secret.yaml
	kubectl delete -f manifests/edc.yaml
	kubectl delete -f manifests/vector-aggregator-discovery.yaml

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

opensearch:
	helm repo add opensearch https://opensearch-project.github.io/helm-charts/ &
	helm repo update &
	helm --version=2.11.3 install opensearch opensearch/opensearch -f ./manifests/opensearch-values.yaml &
	helm --version=2.9.2 install opensearch-dashboards opensearch/opensearch-dashboards -f ./manifests/opensearch-dashboards-values.yaml

vector:
	helm repo add vector https://helm.vector.dev &
	helm repo update &
	helm --version=0.21.0 install vector-aggregator vector/vector -f ./manifests/vector-values.yaml

apply-vector: opensearch vector

init-cluster:
	stackablectl op in secret commons -k
