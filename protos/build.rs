fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("src/hello/hello.proto")?;
    tonic_prost_build::compile_protos("src/event/event.proto")?;

    Ok(())
}
