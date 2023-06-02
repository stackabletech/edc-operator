apply-crd:
	cargo run -- crd | kubectl apply -f -

apply-example:
	kubectl apply -f manifests/edc.yaml &
	kubectl apply -f manifests/vector-aggregator-discovery.yaml

delete-example:
	kubectl delete -f manifests/edc.yaml &
	kubectl delete -f manifests/vector-aggregator-discovery.yaml

secrets:
	kubectl create secret generic connector-cert \
	--from-file=resources/cert.pfx \
	--from-file=resources/vault.properties

apply: secrets apply-crd apply-example

pf:
	kubectl port-forward svc/connector 19191 &
	kubectl port-forward svc/connector 19192 &
	kubectl port-forward svc/connector 19193 &
	kubectl port-forward svc/connector 19194 &
	kubectl port-forward svc/connector 19195 &
	kubectl port-forward svc/connector 19291 &

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
