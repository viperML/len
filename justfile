# -*- mode: just -*-

[private]
default:
    @just -l

doc:
    cargo clean --doc
    cargo doc --verbose --no-deps
    echo '<meta http-equiv=refresh content=0;url=len/index.html>' > target/doc/index.html

insta:
    cargo insta test --delete-unreferenced-snapshots

web:
    wasm-pack build --target web ./len-web --release
    rm -rvf _site
    mkdir -pv _site
    cp -vfr len-web/pkg len-web/index.html len-web/index.js _site