# Fluent-ECS

Fluent ECS is a filter plugin for [fluentbit](https://fluentbit.io/) that aims to transform logs of various sources to the field schema defined by the [Elastic Common Schema](https://www.elastic.co/guide/en/ecs/current/ecs-using-ecs.html).

# Current status

works on my machine

Log transformations for application are added as required for a specific logging setup.
The transformation rules are not exhaustive.
They do not cover every log that may be produced by the supported applications.

Because of the fluent-bit issue [#8156](https://github.com/fluent/fluent-bit/issues/8156) Rust filters like fluent-ecs do not seem to work with the size of real world logs at the moment.
When manually fixing and compiling fluent-bit as described there in the comments, fluent-ecs will work.

# Build

Add the target for web assembly.

    rustup target add wasm32-unknown-unknown

Build with cargo.

    cargo build --target wasm32-unknown-unknown --release

# Handling event.severity

Fluent ECS tries to provide normalized values for the field `field.severity` across different applications.
The following values are used.

| level | severity |
+-------+----------|
| trace |  50      |
| debug | 100      |
| info  | 200      |
| warn  | 300      |
| error | 400      |

# Supported fluent-bit plugins

## Kubernetes
Information added by the [Kubernetes Plugin](https://docs.fluentbit.io/manual/pipeline/filters/kubernetes) are converted to the ECS scheme.

# Supported applications
Fluent ECS tries to detect the application that produced logs in order do convert these logs app-specifically.
At the moment the application detection is based on evaluating labels and annotations added by the fluent-bit Kubernetes plugin.
The following annotations and labels are evaluated.
The first that matches a keyword for a supported applications determines how the log event is processed further.

* Annotation: fluent-ecs.bieniek-it.de/parser
* Label: app.kubernetes.io/name
* Label: component

## etcd
* Keyword: etcd

Etcd logs in JSON format.
The fluent-ecs support for etcd moves JSON fields unknown in ECS to a single array "misc".
This way the log index is not cluttered with to much too etcd-specific fields.

## Metallb
* Keyword: etcd

Metallb logs in JSON format.
The fluent-ecs support for Metallb moves JSON fields unknown in ECS to a single array "misc".
This way the log index is not cluttered with to much too Metallb-specific fields.

## Kubernetes Dashboard
* Keyword: kubernetes-dashboard-metrics-scraper

Some of the logs of the metrics scraper are JSON logs.
The keys are converted to the correct ECS keys.

## Postfix
* Keyword: postfix

Postfix Logs in plain text.
fluent-ecs will parse these plain text logs and will extract information about network connections and transferred mails.