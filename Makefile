run: build
	python3 -m http.server

build:
	wasm-pack build --target web
