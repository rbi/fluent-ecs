use serde_json::Value;

use crate::model::fluentbit::Kubernetes;
use crate::model::FluentBitJson;

pub fn convert_kubernetes_metadata(json: &mut FluentBitJson) {
    let kubernetes = json.kubernetes.take();
    if let Some(kubernetes) = kubernetes {
        convert(kubernetes, json);
        json.kubernetes = Option::None;
    }
}

fn convert(kubernetes: Kubernetes, json: &mut FluentBitJson) {
    // host
    if let Some(host) = kubernetes.host {
        json.host().hostname = Some(host);
    }

    // service
    if let Some(Value::String(env)) = kubernetes.labels.get("app.kubernetes.io/instance") {
        json.service().environment = Some(env.to_string());
    }
    if let Some(Value::String(name)) = kubernetes.labels.get("app.kubernetes.io/name") {
        json.service().name = Some(name.to_string());
    }
    if let Some(Value::String(version)) = kubernetes.labels.get("app.kubernetes.io/version") {
        json.service().version = Some(version.to_string());
    }

    // orchestrator
    let orchestrator = json.orchestrator();
    orchestrator.type_val = Some("kubernetes".to_string());
    orchestrator.namespace = kubernetes.namespace_name;

    let orchestrator_resource = orchestrator.resource();
    orchestrator_resource.id = kubernetes.pod_id;
    orchestrator_resource.name = kubernetes.pod_name;
    orchestrator_resource.type_val = Some("Pod".to_string());
    orchestrator_resource.annotations = to_vec(kubernetes.annotations);
    orchestrator_resource.label = to_vec(kubernetes.labels);

    if let Some(parent_type) = detect_resource_parent(&orchestrator_resource.label) {
        orchestrator_resource.parent().type_val = Some(parent_type);
    }

    // container
    if let Some(docker_id) = kubernetes.docker_id {
        json.container().id = Some(docker_id);
    }
    if let Some(container_name) = kubernetes.container_name {
        json.container().name = Some(container_name);
    }
    if let Some(container_image) = kubernetes.container_image {
        json.container().image().name = Some(container_image);
    }

    let container_hash = kubernetes
        .container_hash
        .map(extract_container_hash)
        .flatten();
    if let Some(hash) = container_hash {
        json.container().image().hash().all = vec![hash]
    }
}

fn extract_container_hash(hash: String) -> Option<String> {
    let parts: Vec<&str> = hash.split("@").collect();
    if parts.len() <= 1 {
        return None;
    }
    parts.last().map(|last_part| last_part.to_string())
}

fn detect_resource_parent(label: &Vec<String>) -> Option<String> {
    label.into_iter().find_map(|candidate| {
        if candidate.starts_with("statefulset.kubernetes.io/pod-name:") {
            Some("StatefulSet".to_string())
        } else {
            None
        }
    })
}

fn to_vec(map: serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    map.into_iter()
        .map(|pair| {
            format!(
                "{}:{}",
                pair.0,
                match pair.1 {
                    serde_json::Value::String(val) => val.to_string(),
                    val => val.to_string(),
                }
            )
        })
        .collect()
}
