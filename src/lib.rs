pub mod benchmarks;
pub mod curve_algos;
pub mod ed25519_eddsa;
pub mod gpu;
pub mod moduli;
pub mod secp256k1_ecdsa;
pub mod secp256r1_ecdsa;
pub mod shader;
pub mod precompute;
pub mod tests;

/// This error is raised if the shader silently fails to execute.
#[derive(Debug, Clone)]
pub struct ShaderFailureError;
