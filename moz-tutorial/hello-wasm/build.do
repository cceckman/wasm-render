
redo-always

wasm-pack build --target web

sha256sum pkg/* | redo-stamp
