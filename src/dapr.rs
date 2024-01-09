#[allow(clippy::module_inception)]
pub mod dapr {
    pub mod proto {
        pub mod common {
            pub mod v1 {
                tonic::include_proto!("dapr.proto.common.v1");
            }
        }
        pub mod runtime {
            pub mod v1 {
                tonic::include_proto!("dapr.proto.runtime.v1");
            }
        }
    }
}
