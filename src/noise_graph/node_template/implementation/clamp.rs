use noise::Clamp;

use crate::noise_graph::graph_ext::NodeEvaluator;
use crate::noise_graph::node_attribute::NodeAttribute;
use crate::noise_graph::node_template::{NodeBuilder, NodeImpl};
use crate::noise_graph::DynNoiseFn;

impl NodeImpl for Clamp<f64, DynNoiseFn, 2> {
    fn build(builder: &mut NodeBuilder) {
        builder
            .input_noise("source")
            .input_f64("bounds lower", -1.0)
            .input_f64("bounds upper", 1.0)
            .output_noise();
    }

    fn evaluate(evaluator: &mut NodeEvaluator) -> anyhow::Result<NodeAttribute> {
        let source = evaluator.get_noise_function("source")?;
        let bounds_lower = evaluator.get_f64("bounds lower")?;
        let bounds_upper = evaluator.get_f64("bounds upper")?;
        let noise = Clamp::new(source).set_bounds(bounds_lower, bounds_upper);
        evaluator.output_noise(noise)
    }
}
