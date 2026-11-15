//! KNIME Workflow Analytical Pipeline Engine for SigmaOS

pub trait KnimeNode {
    fn process(&self, input: &[f64]) -> Vec<f64>;
}

pub struct MinMaxNormalizationNode {
    pub min_target: f64,
    pub max_target: f64,
}

impl KnimeNode for MinMaxNormalizationNode {
    fn process(&self, input: &[f64]) -> Vec<f64> {
        if input.is_empty() {
            return Vec::new();
        }
        let (min, max) = input
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &v| {
                (mn.min(v), mx.max(v))
            });
        if (max - min).abs() < f64::EPSILON {
            return vec![self.min_target; input.len()];
        }
        input
            .iter()
            .map(|&x| {
                let norm = (x - min) / (max - min);
                self.min_target + norm * (self.max_target - self.min_target)
            })
            .collect()
    }
}

pub struct FilterNode {
    pub threshold: f64,
    pub keep_greater: bool,
}

impl KnimeNode for FilterNode {
    fn process(&self, input: &[f64]) -> Vec<f64> {
        input
            .iter()
            .copied()
            .filter(|&x| {
                if self.keep_greater {
                    x > self.threshold
                } else {
                    x <= self.threshold
                }
            })
            .collect()
    }
}

pub struct KnimePipeline {
    nodes: Vec<Box<dyn KnimeNode>>,
}

impl Default for KnimePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl KnimePipeline {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add_node<N: KnimeNode + 'static>(&mut self, node: N) {
        self.nodes.push(Box::new(node));
    }

    pub fn execute(&self, data: &[f64]) -> Vec<f64> {
        let mut current = data.to_vec();
        for node in &self.nodes {
            current = node.process(&current);
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knime_pipeline_execution_aaa() {
        // Arrange
        let mut pipeline = KnimePipeline::new();
        pipeline.add_node(MinMaxNormalizationNode {
            min_target: 0.0,
            max_target: 100.0,
        });
        pipeline.add_node(FilterNode {
            threshold: 50.0,
            keep_greater: true,
        });

        let raw_data = vec![10.0, 20.0, 30.0, 40.0, 50.0];

        // Act
        // 10 -> 0, 20 -> 25, 30 -> 50, 40 -> 75, 50 -> 100. Filter > 50 gives [75.0, 100.0]
        let result = pipeline.execute(&raw_data);

        // Assert
        assert_eq!(result.len(), 2);
        assert!((result[0] - 75.0).abs() < 1e-4);
        assert!((result[1] - 100.0).abs() < 1e-4);
    }
}
