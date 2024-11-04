// This file is @generated by prost-build.
/// HTTPExtension includes HTTP verb and querystring
/// when Dapr runtime delivers HTTP content.
///
/// For example, when callers calls http invoke api
/// `POST <http://localhost:3500/v1.0/invoke/<app_id>/method/<method>?query1=value1&query2=value2`>
///
/// Dapr runtime will parse POST as a verb and extract querystring to quersytring map.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpExtension {
    /// Required. HTTP verb.
    #[prost(enumeration = "http_extension::Verb", tag = "1")]
    pub verb: i32,
    /// Optional. querystring represents an encoded HTTP url query string in the following format: name=value&name2=value2
    #[prost(string, tag = "2")]
    pub querystring: ::prost::alloc::string::String,
}
/// Nested message and enum types in `HTTPExtension`.
pub mod http_extension {
    /// Type of HTTP 1.1 Methods
    /// RFC 7231: <https://tools.ietf.org/html/rfc7231#page-24>
    /// RFC 5789: <https://datatracker.ietf.org/doc/html/rfc5789>
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Verb {
        None = 0,
        Get = 1,
        Head = 2,
        Post = 3,
        Put = 4,
        Delete = 5,
        Connect = 6,
        Options = 7,
        Trace = 8,
        Patch = 9,
    }
    impl Verb {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Self::None => "NONE",
                Self::Get => "GET",
                Self::Head => "HEAD",
                Self::Post => "POST",
                Self::Put => "PUT",
                Self::Delete => "DELETE",
                Self::Connect => "CONNECT",
                Self::Options => "OPTIONS",
                Self::Trace => "TRACE",
                Self::Patch => "PATCH",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "NONE" => Some(Self::None),
                "GET" => Some(Self::Get),
                "HEAD" => Some(Self::Head),
                "POST" => Some(Self::Post),
                "PUT" => Some(Self::Put),
                "DELETE" => Some(Self::Delete),
                "CONNECT" => Some(Self::Connect),
                "OPTIONS" => Some(Self::Options),
                "TRACE" => Some(Self::Trace),
                "PATCH" => Some(Self::Patch),
                _ => None,
            }
        }
    }
}
/// InvokeRequest is the message to invoke a method with the data.
/// This message is used in InvokeService of Dapr gRPC Service and OnInvoke
/// of AppCallback gRPC service.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InvokeRequest {
    /// Required. method is a method name which will be invoked by caller.
    #[prost(string, tag = "1")]
    pub method: ::prost::alloc::string::String,
    /// Required in unary RPCs. Bytes value or Protobuf message which caller sent.
    /// Dapr treats Any.value as bytes type if Any.type_url is unset.
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<::prost_types::Any>,
    /// The type of data content.
    ///
    /// This field is required if data delivers http request body
    /// Otherwise, this is optional.
    #[prost(string, tag = "3")]
    pub content_type: ::prost::alloc::string::String,
    /// HTTP specific fields if request conveys http-compatible request.
    ///
    /// This field is required for http-compatible request. Otherwise,
    /// this field is optional.
    #[prost(message, optional, tag = "4")]
    pub http_extension: ::core::option::Option<HttpExtension>,
}
/// InvokeResponse is the response message including data and its content type
/// from app callback.
/// This message is used in InvokeService of Dapr gRPC Service and OnInvoke
/// of AppCallback gRPC service.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InvokeResponse {
    /// Required in unary RPCs. The content body of InvokeService response.
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<::prost_types::Any>,
    /// Required. The type of data content.
    #[prost(string, tag = "2")]
    pub content_type: ::prost::alloc::string::String,
}
/// Chunk of data sent in a streaming request or response.
/// This is used in requests including InternalInvokeRequestStream.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamPayload {
    /// Data sent in the chunk.
    /// The amount of data included in each chunk is up to the discretion of the sender, and can be empty.
    /// Additionally, the amount of data doesn't need to be fixed and subsequent messages can send more, or less, data.
    /// Receivers must not make assumptions about the number of bytes they'll receive in each chunk.
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// Sequence number. This is a counter that starts from 0 and increments by 1 on each chunk sent.
    #[prost(uint64, tag = "2")]
    pub seq: u64,
}
/// StateItem represents state key, value, and additional options to save state.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateItem {
    /// Required. The state key
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    /// Required. The state data for key
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    /// The entity tag which represents the specific version of data.
    /// The exact ETag format is defined by the corresponding data store.
    #[prost(message, optional, tag = "3")]
    pub etag: ::core::option::Option<Etag>,
    /// The metadata which will be passed to state store component.
    #[prost(map = "string, string", tag = "4")]
    pub metadata: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    /// Options for concurrency and consistency to save the state.
    #[prost(message, optional, tag = "5")]
    pub options: ::core::option::Option<StateOptions>,
}
/// Etag represents a state item version
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Etag {
    /// value sets the etag value
    #[prost(string, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
/// StateOptions configures concurrency and consistency for state operations
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct StateOptions {
    #[prost(enumeration = "state_options::StateConcurrency", tag = "1")]
    pub concurrency: i32,
    #[prost(enumeration = "state_options::StateConsistency", tag = "2")]
    pub consistency: i32,
}
/// Nested message and enum types in `StateOptions`.
pub mod state_options {
    /// Enum describing the supported concurrency for state.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum StateConcurrency {
        ConcurrencyUnspecified = 0,
        ConcurrencyFirstWrite = 1,
        ConcurrencyLastWrite = 2,
    }
    impl StateConcurrency {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Self::ConcurrencyUnspecified => "CONCURRENCY_UNSPECIFIED",
                Self::ConcurrencyFirstWrite => "CONCURRENCY_FIRST_WRITE",
                Self::ConcurrencyLastWrite => "CONCURRENCY_LAST_WRITE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "CONCURRENCY_UNSPECIFIED" => Some(Self::ConcurrencyUnspecified),
                "CONCURRENCY_FIRST_WRITE" => Some(Self::ConcurrencyFirstWrite),
                "CONCURRENCY_LAST_WRITE" => Some(Self::ConcurrencyLastWrite),
                _ => None,
            }
        }
    }
    /// Enum describing the supported consistency for state.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum StateConsistency {
        ConsistencyUnspecified = 0,
        ConsistencyEventual = 1,
        ConsistencyStrong = 2,
    }
    impl StateConsistency {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Self::ConsistencyUnspecified => "CONSISTENCY_UNSPECIFIED",
                Self::ConsistencyEventual => "CONSISTENCY_EVENTUAL",
                Self::ConsistencyStrong => "CONSISTENCY_STRONG",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "CONSISTENCY_UNSPECIFIED" => Some(Self::ConsistencyUnspecified),
                "CONSISTENCY_EVENTUAL" => Some(Self::ConsistencyEventual),
                "CONSISTENCY_STRONG" => Some(Self::ConsistencyStrong),
                _ => None,
            }
        }
    }
}
/// ConfigurationItem represents all the configuration with its name(key).
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigurationItem {
    /// Required. The value of configuration item.
    #[prost(string, tag = "1")]
    pub value: ::prost::alloc::string::String,
    /// Version is response only and cannot be fetched. Store is not expected to keep all versions available
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
    /// the metadata which will be passed to/from configuration store component.
    #[prost(map = "string, string", tag = "3")]
    pub metadata: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
}
