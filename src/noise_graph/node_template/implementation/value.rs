use crate::noise_graph::graph_ext::NodeEvaluator;
use crate::noise_graph::node_attribute::NodeAttribute;
use crate::noise_graph::node_template::{NodeBuilder, NodeImpl};
use noise::Value;

impl NodeImpl for Value {
    fn build(builder: &mut NodeBuilder) {
        builder.output_noise();
    }

    fn evaluate(evaluator: &mut NodeEvaluator) -> anyhow::Result<NodeAttribute> {
        let noise = Value::default();
        evaluator.output_noise(noise)
    }
}
