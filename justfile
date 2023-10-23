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