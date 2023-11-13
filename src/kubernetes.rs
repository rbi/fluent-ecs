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

    // container
    let container = json.container();
    container.id = kubernetes.docker_id;
    container.name = kubernetes.container_name;
    container.image().name = kubernetes.container_image;
}
