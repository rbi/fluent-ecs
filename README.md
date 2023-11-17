# Fluent-ECS

Fluent ECS is a filter plugin for [fluentbit](https://fluentbit.io/) that aims to transform logs of various sources to the field schema defined by the [Elastic Common Schema](https://www.elastic.co/guide/en/ecs/current/ecs-using-ecs.html).

# Build

Add the target for web assembly.

    rustup target add wasm32-unknown-unknown

Build with cargo.

    cargo build --target wasm32-unknown-unknown --release

# event.severity

Fluent ECS tries to provide normalized values for the field `field.severity` across different applications.
The following values are used.

| level | severity |
+-------+----------|
| trace |  50      |
| debug | 100      |
| info  | 200      |
| warn  | 300      |
| error | 400      |