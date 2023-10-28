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

web := "./len-web"

web-build:
    cd {{web}} && npm install --omit optional && npx webpack --mode=production
    cp -vr {{web}}/dist _site

web-dev:
    cd {{web}} && npx webpack-dev-server --open --mode development