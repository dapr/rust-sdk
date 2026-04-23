.PHONY: proto-gen
proto-gen:
	cargo run --bin proto-gen

.PHONY: check-diff-proto
check-diff-proto:
	git diff --exit-code ./proto/ ./dapr/src/dapr/ ./examples/src/invoke/protos/