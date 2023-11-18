pub mod ecs {
    use chrono::{DateTime, FixedOffset};
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
    pub struct Event {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dataset: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub module: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub kind: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub category: Vec<String>,
        #[serde(rename = "type")]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub type_val: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub outcome: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub action: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub created: Option<DateTime<FixedOffset>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub severity: Option<u32>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl Event {
        pub fn new() -> Self {
            Event {
                dataset: None,
                module: None,
                kind: None,
                category: Vec::new(),
                type_val: Vec::new(),
                outcome: None,
                action: None,
                created: None,
                severity: None,
                other: Value::Null,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub message: Option<String>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl Error {
        pub fn new() -> Self {
            Error {
                message: None,
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
    pub struct Log {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub level: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub logger: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub origin: Option<LogOrigin>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl Log {
        pub fn new() -> Self {
            Log {
                level: None,
                logger: None,
                origin: None,
                other: Value::Null,
            }
        }

        pub fn origin(&mut self) -> &mut LogOrigin {
            self.origin.get_or_insert_with(|| LogOrigin::new())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct LogOrigin {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub file: Option<LogOriginFile>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl LogOrigin {
        pub fn new() -> Self {
            LogOrigin {
                file: None,
                other: Value::Null,
            }
        }
        pub fn file(&mut self) -> &mut LogOriginFile {
            self.file.get_or_insert_with(|| LogOriginFile::new())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct LogOriginFile {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub line: Option<u32>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl LogOriginFile {
        pub fn new() -> Self {
            LogOriginFile {
                name: None,
                line: None,
                other: Value::Null,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Network {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub protocol: Option<String>,

        #[serde(flatten)]
        pub other: Value,
    }

    impl Network {
        pub fn new() -> Self {
            Network {
                protocol: None,
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
            self.resource
                .get_or_insert_with(|| OrchestratorResource::new())
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
        pub annotations: Vec<String>,
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
                annotations: Vec::new(),
                label: Vec::new(),
                parent: None,
                other: Value::Null,
            }
        }

        pub fn parent(&mut self) -> &mut OrchestratorResourceParent {
            self.parent
                .get_or_insert_with(|| OrchestratorResourceParent::new())
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
        #[serde(default)]
        #[serde(skip_serializing_if = "Map::is_empty")]
        pub annotations: Map<String, Value>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Map::is_empty")]
        pub labels: Map<String, Value>,

        #[serde(flatten)]
        pub other: Value,
    }
}

use chrono::{DateTime, FixedOffset};
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct FluentBitJson {
    // fluent-bit input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kubernetes: Option<fluentbit::Kubernetes>,

    // ecs output
    #[serde(rename = "@timestamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<EventOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ecs::Container>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<ecs::Host>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log: Option<LogOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<ecs::Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orchestrator: Option<ecs::Orchestrator>,

    // other fields
    #[serde(flatten)]
    pub other: Map<String, Value>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub misc: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventOrString {
    Event(ecs::Event),
    String(String),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorOrString {
    Error(ecs::Error),
    String(String),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogOrString {
    Log(ecs::Log),
    String(String),
}

impl FluentBitJson {
    pub fn container(&mut self) -> &mut ecs::Container {
        self.container.get_or_insert_with(|| ecs::Container::new())
    }
    pub fn host(&mut self) -> &mut ecs::Host {
        self.host.get_or_insert_with(|| ecs::Host::new())
    }
    pub fn network(&mut self) -> &mut ecs::Network {
        self.network.get_or_insert_with(|| ecs::Network::new())
    }
    pub fn orchestrator(&mut self) -> &mut ecs::Orchestrator {
        self.orchestrator
            .get_or_insert_with(|| ecs::Orchestrator::new())
    }

    pub fn log(&mut self) -> &mut ecs::Log {
        match &self.log {
            Some(LogOrString::Log(_)) => (),
            _ => {
                self.log = Some(LogOrString::Log(ecs::Log::new()));
                ()
            }
        };

        match &mut self.log {
            Some(LogOrString::Log(event)) => event,
            _ => unreachable!(),
        }
    }

    pub fn event(&mut self) -> &mut ecs::Event {
        match &self.event {
            Some(EventOrString::Event(_)) => (),
            _ => {
                self.event = Some(EventOrString::Event(ecs::Event::new()));
                ()
            }
        };

        match &mut self.event {
            Some(EventOrString::Event(event)) => event,
            _ => unreachable!(),
        }
    }

    pub fn error(&mut self) -> &mut ecs::Error {
        match &self.error {
            Some(ErrorOrString::Error(_)) => (),
            _ => {
                self.error = Some(ErrorOrString::Error(ecs::Error::new()));
                ()
            }
        };

        match &mut self.error {
            Some(ErrorOrString::Error(error)) => error,
            _ => unreachable!(),
        }
    }

    pub fn move_key_to_misc(&mut self, key: &str) {
        match self.other.remove(key) {
            Some(val) => self.misc.push(format!("{}:{}", key, val_to_string(val))),
            _ => {}
        }
    }
}

fn val_to_string(val: Value) -> String {
    match val {
        Value::String(val) => val,
        Value::Array(arr) => arr
            .into_iter()
            .map(val_to_string)
            .collect::<Vec<String>>()
            .join(","),
        _ => val.to_string(),
    }
}
