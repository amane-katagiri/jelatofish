# jelatofish

Yet another random image generator for wallpaper or avatar inspired by xstarfish.

## Run in browser

1. Build wasm

```bash
mkdir -p ./build
wasm-pack build --target web
cp web/html/index.html ./build/
cp -r ./pkg/ ./build/
```

1. Run nginx docker container

```bash
docker run --rm -p 8080:80 -v (pwd)/build:/usr/share/nginx/html:ro nginx
```

1. Open [`localhost:8080`](http://localhost:8080/)
