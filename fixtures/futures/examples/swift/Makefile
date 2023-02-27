build:
	cargo build --release
	../../../../target/release/uniffi-fixtures-futures generate --lib-file ../../../../target/release/libuniffi_futures.dylib ../../src/uniffi_futures.udl --language swift --out-dir ../../../../target/release/
	swiftc \
		-module-name uniffi_futures \
		-emit-library -o ../../../../target/release/libuniffi_futures.dylib \
		-emit-module -emit-module-path ../../../../target/release/ \
		-parse-as-library \
		-L ../../../../target/release/ \
		-luniffi_futures \
		-Xcc -fmodule-map-file=../../../../target/release/uniffi_futuresFFI.modulemap \
		../../../../target/release/uniffi_futures.swift
	# There is no equivalent to `-Xcc -fmodule-map-file` in `Package.swift`. The
	# only solution is to rename the modulemap to… `module.modulemap`, and boom,
	# magic.
	cp ../../../../target/release/uniffi_futuresFFI.modulemap ../../../../target/release/module.modulemap

run:
	swift run
