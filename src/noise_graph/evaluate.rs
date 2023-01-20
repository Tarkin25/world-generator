use egui_node_graph::NodeId;
use noise::{MultiFractal, NoiseFn, Perlin, RidgedMulti, ScaleBias, Simplex, ScalePoint};

use crate::noise_graph::{node_template::NodeTemplate, DynNoiseFn};

use super::{
    node_value::{NodeValue, NoiseType, Operator},
    MyGraph, OutputsCache,
};

/// Recursively evaluates all dependencies of this node, then evaluates the node itself.
pub fn evaluate_node(
    graph: &MyGraph,
    node_id: NodeId,
    outputs_cache: &mut OutputsCache,
) -> anyhow::Result<NodeValue> {
    let node = &graph[node_id];
    let mut evaluator = Evaluator::new(graph, outputs_cache, node_id);
    match node.user_data.template {
        NodeTemplate::Number => {
            let value = evaluator.get_f64("value")?;
            evaluator.output_number(value)
        },
        NodeTemplate::Arithmetic => {
            let operator = evaluator.get_operator("operator")?;
            let a = evaluator.get_f64("a")?;
            let b = evaluator.get_f64("b")?;
            evaluator.output_number(operator.apply(a, b))
        },
        NodeTemplate::Perlin => evaluator.output_noise(Perlin::default()),
        NodeTemplate::ScaleBias => {
            let scale = evaluator.get_f64("scale")?;
            let bias = evaluator.get_f64("bias")?;
            let source = evaluator.get_noise_function("source")?;
            let noise = ScaleBias::new(source.clone())
                .set_scale(scale)
                .set_bias(bias);
            evaluator.output_noise(noise)
        },
        NodeTemplate::ScalePoint => {
            let source = evaluator.get_noise_function("source")?;
            let x = evaluator.get_f64("x")?;
            let y = evaluator.get_f64("y")?;
            let z = evaluator.get_f64("z")?;
            let u = evaluator.get_f64("u")?;
            let noise = ScalePoint::new(source)
            .set_all_scales(x, y, z, u);
            evaluator.output_noise(noise)
        },
        NodeTemplate::RidgedMulti => {
            let octaves = evaluator.get_usize("octaves")?;
            let frequency = evaluator.get_f64("frequency")?;
            let lacunarity = evaluator.get_f64("lacunarity")?;
            let persistence = evaluator.get_f64("persistence")?;
            let attenuation = evaluator.get_f64("attenuation")?;

            match evaluator.get_noise_type()? {
                NoiseType::Perlin => {
                    let noise = RidgedMulti::<Perlin>::default()
                        .set_octaves(octaves)
                        .set_frequency(frequency)
                        .set_lacunarity(lacunarity)
                        .set_persistence(persistence)
                        .set_attenuation(attenuation);
                    evaluator.output_noise(noise)
                },
                NoiseType::Simplex => {
                    let noise = RidgedMulti::<Simplex>::default()
                        .set_octaves(octaves)
                        .set_frequency(frequency)
                        .set_lacunarity(lacunarity)
                        .set_persistence(persistence)
                        .set_attenuation(attenuation);
                    evaluator.output_noise(noise)
                }
            }
        }
    }
}

fn populate_output(
    graph: &MyGraph,
    outputs_cache: &mut OutputsCache,
    node_id: NodeId,
    param_name: &str,
    value: NodeValue,
) -> anyhow::Result<NodeValue> {
    let output_id = graph[node_id].get_output(param_name)?;
    outputs_cache.insert(output_id, value.clone());
    Ok(value)
}

// Evaluates the input value of
fn evaluate_input(
    graph: &MyGraph,
    node_id: NodeId,
    param_name: &str,
    outputs_cache: &mut OutputsCache,
) -> anyhow::Result<NodeValue> {
    let input_id = graph[node_id].get_input(param_name)?;

    // The output of another node is connected.
    if let Some(other_output_id) = graph.connection(input_id) {
        // The value was already computed due to the evaluation of some other
        // node. We simply return value from the cache.
        if let Some(other_value) = outputs_cache.get(&other_output_id) {
            Ok(other_value.clone())
        }
        // This is the first time encountering this node, so we need to
        // recursively evaluate it.
        else {
            // Calling this will populate the cache
            evaluate_node(graph, graph[other_output_id].node, outputs_cache)?;

            // Now that we know the value is cached, return it
            Ok(outputs_cache
                .get(&other_output_id)
                .expect("Cache should be populated")
                .clone())
        }
    }
    // No existing connection, take the inline value instead.
    else {
        Ok(graph[input_id].value.clone())
    }
}

// To solve a similar problem as creating node types above, we define an
// Evaluator as a convenience. It may be overkill for this small example,
// but something like this makes the code much more readable when the
// number of nodes starts growing.

struct Evaluator<'a> {
    graph: &'a MyGraph,
    outputs_cache: &'a mut OutputsCache,
    node_id: NodeId,
}
impl<'a> Evaluator<'a> {
    fn new(graph: &'a MyGraph, outputs_cache: &'a mut OutputsCache, node_id: NodeId) -> Self {
        Self {
            graph,
            outputs_cache,
            node_id,
        }
    }
    fn evaluate_input(&mut self, name: &str) -> anyhow::Result<NodeValue> {
        // Calling `evaluate_input` recursively evaluates other nodes in the
        // graph until the input value for a paramater has been computed.
        evaluate_input(self.graph, self.node_id, name, self.outputs_cache)
    }
    fn populate_output(&mut self, name: &str, value: NodeValue) -> anyhow::Result<NodeValue> {
        // After computing an output, we don't just return it, but we also
        // populate the outputs cache with it. This ensures the evaluation
        // only ever computes an output once.
        //
        // The return value of the function is the "final" output of the
        // node, the thing we want to get from the evaluation. The example
        // would be slightly more contrived when we had multiple output
        // values, as we would need to choose which of the outputs is the
        // one we want to return. Other outputs could be used as
        // intermediate values.
        //
        // Note that this is just one possible semantic interpretation of
        // the graphs, you can come up with your own evaluation semantics!
        populate_output(self.graph, self.outputs_cache, self.node_id, name, value)
    }
    fn get_f64(&mut self, name: &str) -> anyhow::Result<f64> {
        self.evaluate_input(name)?.try_to_f64()
    }
    fn get_usize(&mut self, name: &str) -> anyhow::Result<usize> {
        self.evaluate_input(name)?.try_to_usize()
    }
    fn get_noise_function(&mut self, name: &str) -> anyhow::Result<DynNoiseFn> {
        self.evaluate_input(name)?.try_to_noise_function()
    }
    fn get_noise_type(&mut self) -> anyhow::Result<NoiseType> {
        self.evaluate_input("noise type")?.try_to_noise_type()
    }
    fn get_operator(&mut self, name: &str) -> anyhow::Result<Operator> {
        self.evaluate_input(name)?.try_to_operator()
    }
    fn output_noise(
        &mut self,
        noise: impl NoiseFn<f64, 2> + Send + Sync + 'static,
    ) -> anyhow::Result<NodeValue> {
        self.populate_output("out", NodeValue::NoiseFunction(DynNoiseFn::new(noise)))
    }
    fn output_number(
        &mut self,
        value: f64,
    ) -> anyhow::Result<NodeValue> {
        self.populate_output("out", NodeValue::F64(value))
    }
}