## More port from rust-qt-binding-generator

This directory contains patches that can be applied to port example
using rust-qt-binding-generator to this crate.

The goal of the patch is to attempt to show that using this crate is
simpler than using the binding generator.

### qrep

Is the tool presented here:
https://www.vandenoever.info/blog/2018/10/30/building_qt_apps_with_cargo.html

To apply the patch and run the program:

```
git clone https://anongit.kde.org/scratch/vandenoever/qrep
cd qrep
git checkout bdbde040e74819351609581c0d98a59bbfeecbf9 -b qmetaobject-rs
git am ../qmetaobject-rs/examples/rqbg/qrep.patch
cargo run
```

The port does the same as the original.
Contrary to the original, there is no need to write a single line of C++.
And even implementations.rs has less lines than before.

### mailmodel

Mailmodel was introduced here
https://www.vandenoever.info/blog/2018/09/16/browsing_your_mail_with_rust_and_qt.html

To apply the patch and run the program:

```
git clone https://anongit.kde.org/scratch/vandenoever/mailmodel
cd mailmodel
git checkout 87991f1090b57706f5c713c8425684eba144cec2 -b qmetaobject-rs
git am ../qmetaobject-rs/examples/rqbg/mailmodel.patch
cat README.md
# create a configuration file as explained
cargo run config.json
```

Note: If you get compilation error because of openssl, try setting these environment variable:
`OPENSSL_LIB_DIR=/usr/lib/openssl-1.0 OPENSSL_INCLUDE_DIR=/usr/include/openssl-1.0`

