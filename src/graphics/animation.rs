use crate::geometry::{Interpolable, Quaternion, Vec3};
pub struct Animation {
    pub channels: Vec<AnimationChannel>,
}

impl<'a> Animation {
    pub fn compute(&'a self, t: f32) -> impl Iterator<Item = AnimatedValue> + 'a {
        self.channels.iter().map(move |c| c.compute(t))
    }
}

pub enum AnimatedProperty {
    Translation(Sampler<Vec3>),
    Rotation(Sampler<Quaternion>),
    Scale(Sampler<Vec3>),
}

impl AnimatedProperty {
    fn get_value(&self, index: usize, node_id: usize) -> AnimatedValue {
        match self {
            Self::Translation(sampler) => {
                AnimatedValue::Translation(node_id, sampler.get_value(index))
            }
            Self::Rotation(sampler) => AnimatedValue::Rotation(node_id, sampler.get_value(index)),
            Self::Scale(sampler) => AnimatedValue::Scale(node_id, sampler.get_value(index)),
        }
    }

    fn interpolate_value(
        &self,
        index: usize,
        node_id: usize,
        t: f32,
        t_min: f32,
        t_max: f32,
    ) -> AnimatedValue {
        match self {
            Self::Translation(sampler) => AnimatedValue::Translation(
                node_id,
                sampler.interpolate_value(index, t, t_min, t_max),
            ),
            Self::Rotation(sampler) => {
                AnimatedValue::Rotation(node_id, sampler.interpolate_value(index, t, t_min, t_max))
            }
            Self::Scale(sampler) => {
                AnimatedValue::Scale(node_id, sampler.interpolate_value(index, t, t_min, t_max))
            }
        }
    }
}

pub enum AnimatedValue {
    Translation(usize, Vec3),
    Rotation(usize, Quaternion),
    Scale(usize, Vec3),
}

pub struct AnimationChannel {
    pub node_id: usize,
    pub animated_property: AnimatedProperty,
    pub timestamps: Vec<f32>,
    pub t_min: f32,
    pub t_max: f32,
}

impl AnimationChannel {
    fn compute(&self, t: f32) -> AnimatedValue {
        if t <= self.t_min {
            return self.animated_property.get_value(0, self.node_id);
        }
        if t >= self.t_max {
            return self
                .animated_property
                .get_value(self.timestamps.len() - 1, self.node_id);
        }
        let index = self.get_index(t);
        self.animated_property.interpolate_value(
            index,
            self.node_id,
            t,
            self.timestamps[index],
            self.timestamps[index + 1],
        )
    }

    fn get_index(&self, t: f32) -> usize {
        let mut index_min = 0;
        let mut index_max = self.timestamps.len() - 1;
        while index_max > index_min + 1 {
            let index_mean = (index_min + index_max) / 2;
            if self.timestamps[index_mean] > t {
                index_max = index_mean;
            } else {
                index_min = index_mean;
            }
        }
        index_min
    }
}

pub enum Sampler<T: Interpolable + Copy> {
    Step(Vec<T>),
    Linear(Vec<T>),
    Cubic(Vec<T>, Vec<T>, Vec<T>),
}

impl<T: Interpolable + Copy> Sampler<T> {
    fn get_value(&self, index: usize) -> T {
        match self {
            Sampler::Cubic(_, values, _) => values[index],
            Sampler::Step(values) => values[index],
            Sampler::Linear(values) => values[index],
        }
    }

    fn interpolate_value(&self, index: usize, t: f32, t_min: f32, t_max: f32) -> T {
        match self {
            Sampler::Step(values) => values[index],
            Sampler::Linear(values) => {
                let alpha = (t - t_min) / (t_max - t_min);
                values[index].linear_interpolation(values[index + 1], alpha)
            }
            Sampler::Cubic(in_tangents, values, out_tangents) => {
                let time_interval = t_max - t_min;
                let alpha = (t - t_min) / time_interval;
                let out_tangent = out_tangents[index];
                let in_tangent = in_tangents[index + 1];
                values[index].cubic_interpolation(
                    values[index + 1],
                    out_tangent,
                    in_tangent,
                    time_interval,
                    alpha,
                )
            }
        }
    }
}
