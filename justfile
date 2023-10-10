
[private]
default:
    @just -l

doc:
    cargo doc
    echo '<meta http-equiv=refresh content=0;url=YOURLIBNAME/index.html>' > target/doc/index.html