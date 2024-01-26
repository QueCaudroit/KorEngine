use crate::{
    geometry::{Quaternion, Transform, Vec3},
    graphics::animation::{AnimatedValue, Animation},
};

pub struct Animator {
    nodes: Vec<Node>,
    start_nodes: Vec<Node>,
    inverse_transforms: Vec<Transform>,
    parents: Vec<usize>,
    pub animations: Vec<Animation>,
}

impl Animator {
    pub fn new(
        all_nodes: &[gltf::Node],
        node_ids: &[usize],
        inverse_transform_option: Option<Vec<Transform>>,
    ) -> (Self, Vec<usize>, Vec<usize>) {
        let mut global_id_to_joint = vec![usize::MAX; all_nodes.len()];
        let mut global_id_to_inner = vec![usize::MAX; all_nodes.len()];
        let mut all_parents = vec![usize::MAX; all_nodes.len()];
        let mut joint_id_to_inner = vec![usize::MAX; node_ids.len()];
        let mut parents = vec![usize::MAX; node_ids.len()];
        let mut start_nodes = Vec::with_capacity(node_ids.len());
        let mut inverse_transforms = Vec::with_capacity(node_ids.len());
        for (i, &node_id) in node_ids.iter().enumerate() {
            global_id_to_joint[node_id] = i;
        }
        for node in all_nodes {
            let parent_id = node.index();
            for child_id in node.children().map(|n| n.index()) {
                all_parents[child_id] = parent_id;
            }
        }
        let mut root_node_id = node_ids[0];
        while all_parents[root_node_id] != usize::MAX {
            root_node_id = all_parents[root_node_id];
        }
        let mut node_to_traverse = vec![root_node_id];
        while let Some(node_id) = node_to_traverse.pop() {
            let joint_id = global_id_to_joint[node_id];
            for child_id in all_nodes[node_id].children().map(|n| n.index()) {
                node_to_traverse.push(child_id);
            }
            if joint_id < usize::MAX {
                let inner_id = start_nodes.len();
                joint_id_to_inner[joint_id] = inner_id;
                global_id_to_inner[node_id] = inner_id;
                let parent_id = global_id_to_inner[all_parents[node_id]];
                parents[inner_id] = parent_id;
                let (translation, rotation, scale) = all_nodes[node_id].transform().decomposed();
                start_nodes.push(Node {
                    rotation: rotation.into(),
                    translation: translation.into(),
                    scale: scale.into(),
                });
                inverse_transforms.push(match &inverse_transform_option {
                    Some(transforms) => transforms[joint_id],
                    None => {
                        let transform =
                            Transform::from_trs(translation.into(), rotation.into(), scale.into())
                                .reverse();
                        if parent_id < usize::MAX {
                            transform.compose(&inverse_transforms[parent_id])
                        } else {
                            transform
                        }
                    }
                })
            }
        }
        (
            Animator {
                nodes: start_nodes.clone(),
                start_nodes,
                inverse_transforms,
                parents,
                animations: Vec::new(),
            },
            global_id_to_inner,
            joint_id_to_inner,
        )
    }

    pub fn reset(&mut self) {
        self.nodes.clone_from_slice(&self.start_nodes);
    }

    pub fn compute_transforms(&self) -> Vec<Transform> {
        let mut cumulated_transforms = Vec::with_capacity(self.nodes.len());
        let mut result = Vec::with_capacity(self.nodes.len());
        let transform = Transform::from_trs(
            self.nodes[0].translation,
            self.nodes[0].rotation,
            self.nodes[0].scale,
        );
        cumulated_transforms.push(transform);
        result.push(transform.compose(&self.inverse_transforms[0]));
        for i in 1..self.nodes.len() {
            let transform = cumulated_transforms[self.parents[i]].compose(&Transform::from_trs(
                self.nodes[i].translation,
                self.nodes[i].rotation,
                self.nodes[i].scale,
            ));
            cumulated_transforms.push(transform);
            result.push(transform.compose(&self.inverse_transforms[i]));
        }
        result
    }

    pub fn scale_node(&mut self, node_id: usize, scale: Vec3) {
        self.nodes[node_id].scale = self.nodes[node_id].scale * scale;
    }

    pub fn translate_node(&mut self, node_id: usize, translation: Vec3) {
        self.nodes[node_id].translation = self.nodes[node_id].translation + translation;
    }

    pub fn rotate_node(&mut self, node_id: usize, rotation: Quaternion) {
        self.nodes[node_id].rotation = rotation * self.nodes[node_id].rotation;
    }

    pub fn animate(&mut self, id: usize, t: f32) {
        for animated_value in self.animations[id].compute(t) {
            match animated_value {
                AnimatedValue::Translation(node_id, translation) => {
                    self.nodes[node_id].translation = translation
                }
                AnimatedValue::Rotation(node_id, rotation) => {
                    self.nodes[node_id].rotation = rotation
                }
                AnimatedValue::Scale(node_id, scale) => self.nodes[node_id].scale = scale,
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Node {
    rotation: Quaternion,
    translation: Vec3,
    scale: Vec3,
}
