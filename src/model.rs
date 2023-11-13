pub mod ecs {
    use serde_derive::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize)]
    pub struct Container {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub image: Option<ContainerImage>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl Container {
        pub fn new() -> Self {
            Container {
                id: None,
                name: None,
                image: None,
                other: Value::Null,
            }
        }

        pub fn image(&mut self) -> &mut ContainerImage {
            self.image.get_or_insert_with(|| ContainerImage::new())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct ContainerImage {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        #[serde(flatten)]
        pub other: Value,
    }
    impl ContainerImage {
        pub fn new() -> Self {
            ContainerImage {
                name: None,
                other: Value::Null,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Host {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub hostname: Option<String>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl Host {
        pub fn new() -> Self {
            Host {
                hostname: None,
                other: Value::Null,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Orchestrator {
        #[serde(rename = "type")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub type_val: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub namespace: Option<String>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl Orchestrator {
        pub fn new() -> Self {
            Orchestrator {
                type_val: None,
                namespace: None,
                other: Value::Null,
            }
        }
    }
}

pub mod fluentbit {
    use serde_derive::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize)]
    pub struct Kubernetes {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub container_image: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub container_hash: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub container_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub docker_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub host: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub namespace_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub pod_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub pod_name: Option<String>,

        #[serde(flatten)]
        pub other: Value,
    }
}

use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct FluentBitJson {
    // fluent-bit input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kubernetes: Option<fluentbit::Kubernetes>,

    // ecs output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ecs::Container>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<ecs::Host>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orchestrator: Option<ecs::Orchestrator>,

    #[serde(flatten)]
    pub other: Value,
}

impl FluentBitJson {
    pub fn container(&mut self) -> &mut ecs::Container {
        self.container.get_or_insert_with(|| ecs::Container::new())
    }
    pub fn host(&mut self) -> &mut ecs::Host {
        self.host.get_or_insert_with(|| ecs::Host::new())
    }
    pub fn orchestrator(&mut self) -> &mut ecs::Orchestrator {
        self.orchestrator
            .get_or_insert_with(|| ecs::Orchestrator::new())
    }
}
