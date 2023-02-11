doc:
	cargo rustdoc --open --all-features --target=wasm32-unknown-unknown -- --cfg docsrs
test:
	wasm-pack test --firefox --test dom