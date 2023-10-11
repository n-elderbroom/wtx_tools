cargo build --release && \
cbindgen -c cbindgen.toml --output cpp/wtx_tools.h && \
clang++ cpp/test.cpp -o cpp/test.out -L ./target/release/ -I ./cpp/ -lwtx_tools
