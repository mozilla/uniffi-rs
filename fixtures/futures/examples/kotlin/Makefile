install-jar:
	curl https://repo1.maven.org/maven2/net/java/dev/jna/jna/5.8.0/jna-5.8.0.jar -o jna.jar
	curl https://repo1.maven.org/maven2/org/jetbrains/kotlinx/kotlinx-coroutines-core-jvm/1.6.4/kotlinx-coroutines-core-jvm-1.6.4.jar -o kotlinx-coroutines-core-jvm.jar

build:
	cargo build --release
	../../../../target/release/uniffi-bindgen generate --lib-file ../../../../target/release/libuniffi_futures.dylib ../../src/uniffi_futures.udl --language kotlin --out-dir .
	rm -f libuniffi_futures.dylib
	ln -s ../../../../target/release/libuniffi_futures.dylib libuniffi_futures.dylib
	kotlinc -cp kotlinx-coroutines-core-jvm.jar:jna.jar -include-runtime -d demo.jar uniffi/fixture/futures/*.kt demo.kt

run:
	java -cp demo.jar:kotlinx-coroutines-core-jvm.jar:jna.jar my.demo.DemoKt
