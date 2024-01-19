use crate::geometry::{Quaternion, Transform, Vec3};

pub struct Animator {
    nodes: Vec<Node>,
    start_nodes: Vec<Node>,
    inverse_transforms: Vec<Transform>,
    parents: Vec<usize>,
}

impl Animator {
    pub fn new(
        all_nodes: &Vec<gltf::Node>,
        root_node_id: usize,
        node_ids: Vec<usize>,
        inverse_transform_option: Option<Vec<Transform>>,
    ) -> (Self, Vec<u32>) {
        let mut global_id_to_joint = vec![usize::MAX; all_nodes.len()];
        let mut global_id_to_inner = vec![usize::MAX; all_nodes.len()];
        let mut all_parents = vec![usize::MAX; all_nodes.len()];
        let mut parents = vec![usize::MAX; node_ids.len()];
        let mut start_nodes = Vec::with_capacity(node_ids.len());
        let mut inverse_transforms = Vec::with_capacity(node_ids.len());
        let mut node_to_traverse = vec![root_node_id];
        for (i, &node_id) in node_ids.iter().enumerate() {
            global_id_to_joint[node_id] = i;
        }
        while let Some(node_id) = node_to_traverse.pop() {
            let joint_id = global_id_to_joint[node_id];
            for child_id in all_nodes[node_id].children().map(|n| n.index()) {
                node_to_traverse.push(child_id);
                if joint_id < usize::MAX && global_id_to_joint[child_id] < usize::MAX {
                    all_parents[child_id] = node_id;
                }
            }
            if joint_id < usize::MAX {
                let inner_id = start_nodes.len();
                let parent_id = global_id_to_inner[all_parents[node_id]];
                global_id_to_inner[node_id] = inner_id;
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
            },
            global_id_to_inner
                .into_iter()
                .map(|i| if i != usize::MAX { i as u32 } else { 0 })
                .collect(),
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
}

#[derive(Clone, Copy)]
pub struct Node {
    rotation: Quaternion,
    translation: Vec3,
    scale: Vec3,
}
