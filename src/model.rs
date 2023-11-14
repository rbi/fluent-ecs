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
        #[serde(skip_serializing_if = "Option::is_none")]
        pub hash: Option<ContainerImageHash>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl ContainerImage {
        pub fn new() -> Self {
            ContainerImage {
                name: None,
                hash: None,
                other: Value::Null,
            }
        }

        pub fn hash(&mut self) -> &mut ContainerImageHash {
            self.hash.get_or_insert_with(|| ContainerImageHash::new())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct ContainerImageHash {
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub all: Vec<String>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl ContainerImageHash {
        pub fn new() -> Self {
            ContainerImageHash {
                all: Vec::new(),
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
        #[serde(skip_serializing_if = "Option::is_none")]
        pub resource: Option<OrchestratorResource>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl Orchestrator {
        pub fn new() -> Self {
            Orchestrator {
                type_val: None,
                namespace: None,
                resource: None,
                other: Value::Null,
            }
        }

        pub fn resource(&mut self) -> &mut OrchestratorResource {
            self.resource.get_or_insert_with(|| OrchestratorResource::new())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct OrchestratorResource {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(rename = "type")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub type_val: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub label: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parent: Option<OrchestratorResourceParent>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl OrchestratorResource {
        pub fn new() -> Self {
            OrchestratorResource {
                id: None,
                name: None,
                type_val: None,
                label: Vec::new(),
                parent: None,
                other: Value::Null,
            }
        }

        pub fn parent(&mut self) -> &mut OrchestratorResourceParent {
            self.parent.get_or_insert_with(|| OrchestratorResourceParent::new())
        }
    }
    #[derive(Serialize, Deserialize)]
    pub struct OrchestratorResourceParent {
        #[serde(rename = "type")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub type_val: Option<String>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl OrchestratorResourceParent {
        pub fn new() -> Self {
            OrchestratorResourceParent {
                type_val: None,
                other: Value::Null,
            }
        }
    }
}

pub mod fluentbit {
    use serde_derive::{Deserialize, Serialize};
    use serde_json::{Map, Value};

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
        #[serde(skip_serializing_if = "Map::is_empty")]
        pub labels: Map<String, Value>,

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
