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
    json.host().hostname = kubernetes.host;

    // orchestrator
    let orchestrator = json.orchestrator();
    orchestrator.type_val = Some("kubernetes".to_string());
    orchestrator.namespace = kubernetes.namespace_name;

    let orchestrator_resource = orchestrator.resource();
    orchestrator_resource.id = kubernetes.pod_id;
    orchestrator_resource.name = kubernetes.pod_name;
    orchestrator_resource.type_val = Some("Pod".to_string());
    orchestrator_resource.label = kubernetes
        .labels
        .into_iter()
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
        .collect();
    if let Some(parent_type) = detect_resource_parent(&orchestrator_resource.label) {
        orchestrator_resource.parent().type_val = Some(parent_type);
    }

    // container
    let container = json.container();
    container.id = kubernetes.docker_id;
    container.name = kubernetes.container_name;
    container.image().name = kubernetes.container_image;

    let container_hash = kubernetes
        .container_hash
        .map(extract_container_hash)
        .flatten();
    if let Some(hash) = container_hash {
        container.image().hash().all = vec![hash]
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
