cargo build && \
cbindgen -c cbindgen.toml --output cpp/wtx_tools.h && \
clang++ cpp/test.cpp -o cpp/test.out -L ./target/debug/ -I ./cpp/ -lwtx_tools
