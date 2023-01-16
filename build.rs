use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &["ola/common/protocol/Ola.proto", "ola/common/rpc/Rpc.proto"],
        &["ola/"],
    )
}
