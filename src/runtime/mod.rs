#[cfg(target_feature = "atomics")]
compile_error!("not yet implemented");
#[cfg(not(target_feature = "atomics"))]
flat_mod! { singlethread }