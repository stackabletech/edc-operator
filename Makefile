apply-crd:
	cargo run -- crd | kubectl apply -f -

apply-example:
	kubectl apply -f manifests/edc.yaml

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

