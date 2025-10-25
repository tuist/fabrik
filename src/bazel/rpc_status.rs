/// Simple definition of google.rpc.Status that matches the proto definition
/// This avoids complex path issues in generated code
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Status {
    /// The status code
    #[prost(int32, tag = "1")]
    pub code: i32,

    /// A developer-facing error message
    #[prost(string, tag = "2")]
    pub message: String,

    /// A list of messages that carry the error details
    #[prost(message, repeated, tag = "3")]
    pub details: ::prost::alloc::vec::Vec<::prost_types::Any>,
}
