use std::sync::Arc;

use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::{
        StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo,
    },
    device::Device,
    memory::allocator::StandardMemoryAllocator,
};

pub struct AllocatorCollection {
    pub memory: Arc<StandardMemoryAllocator>,
    pub command_buffer: StandardCommandBufferAllocator,
    pub descriptor_set: StandardDescriptorSetAllocator,
}

impl AllocatorCollection {
    pub fn new(device: Arc<Device>) -> Self {
        AllocatorCollection {
            memory: Arc::new(StandardMemoryAllocator::new_default(device.clone())),
            command_buffer: StandardCommandBufferAllocator::new(device.clone(), Default::default()),
            descriptor_set: StandardDescriptorSetAllocator::new(
                device,
                StandardDescriptorSetAllocatorCreateInfo {
                    ..Default::default()
                },
            ),
        }
    }
}
