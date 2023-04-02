use std::io::Result;

fn main() -> Result<()> {
    #[cfg(feature = "protobuf")]
    prost_build::compile_protos(&["proto/profile_config.proto"], &["proto/"])?;
    Ok(())
}
