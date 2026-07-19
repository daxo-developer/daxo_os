import Lake
open Lake DSL

package frame_verify where
  version := "0.1.0"
  keywords := #["verification", "os", "kernel"]

@[default_target]
lean_lib FrameVerify where
  srcDir := "."
  roots := #[`FrameVerify]

