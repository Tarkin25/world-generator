use noise::Add;

use crate::noise_graph::{
    graph_ext::NodeEvaluator,
    node_attribute::NodeAttribute,
    node_template::{NodeBuilder, NodeImpl},
    DynNoiseFn,
};

impl NodeImpl for Add<f64, DynNoiseFn, DynNoiseFn, 2> {
    fn build(builder: &mut NodeBuilder) {
        builder
            .input_noise("source 1")
            .input_noise("source 2")
            .output_noise();
    }

    fn evaluate(evaluator: &mut NodeEvaluator) -> anyhow::Result<NodeAttribute> {
        let source_1 = evaluator.get_noise_function("source 1")?;
        let source_2 = evaluator.get_noise_function("source 2")?;
        let noise = noise::Add::new(source_1, source_2);
        evaluator.output_noise(noise)
    }
}
