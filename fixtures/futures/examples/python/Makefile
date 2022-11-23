build:
	cargo build --release
	../../../../target/release/uniffi-bindgen generate --lib-file ../../../../target/release/libuniffi_futures.dylib ../../src/uniffi_futures.udl --language python --out-dir .
	rm -f libuniffi_futures.dylib
	ln -s ../../../../target/release/libuniffi_futures.dylib libuniffi_futures.dylib

run:
	python3 demo.py