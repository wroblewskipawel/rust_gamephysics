use super::{CommandPools, CommandType, Device, PhysicalDeviceConfig, Queues};
use crate::renderer::{mesh::Vertex, Mesh};
use ash::{prelude::VkResult, vk};
use bytemuck::Pod;
use std::{collections::HashSet, iter::FromIterator, mem::size_of, ptr::copy_nonoverlapping};

pub struct MeshOffset {
    pub index_offset: usize,
    pub vertex_offset: usize,
    pub index_count: usize,
}

pub struct MeshData {
    memory: vk::DeviceMemory,
    buffer: vk::Buffer,
    index_offset: usize,
    vertex_offset: usize,
    pub(super) mesh_offsets: Vec<MeshOffset>,
}

pub struct StagingBuffer<'a> {
    memory: vk::DeviceMemory,
    buffer: vk::Buffer,
    fence: vk::Fence,
    device: &'a ash::Device,
}

impl<'a> Device {
    pub(super) fn load_mesh_data(
        device: &ash::Device,
        config: &PhysicalDeviceConfig,
        command_pools: &CommandPools,
        queues: &Queues,
        meshes: &[Mesh],
    ) -> VkResult<MeshData> {
        let mut mesh_offsets = Vec::new();
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u32>::new();
        for mesh in meshes {
            mesh_offsets.push(MeshOffset {
                index_offset: indices.len(),
                vertex_offset: vertices.len(),
                index_count: mesh.indices.len(),
            });
            vertices.extend(mesh.vertices.iter());
            indices.extend(mesh.indices.iter());
        }

        let vertex_byte_size = vertices.len() * size_of::<Vertex>();
        let index_byte_size = indices.len() * size_of::<u32>();
        let buffer_byte_size = vertex_byte_size + index_byte_size;
        let staging_byte_size = usize::max(vertex_byte_size, index_byte_size);

        let queue_indices: Vec<_> = HashSet::<u32>::from_iter([
            config.queue_families.graphics,
            config.queue_families.transfer,
        ])
        .into_iter()
        .collect();

        let buffer = unsafe {
            device.create_buffer(
                &vk::BufferCreateInfo::builder()
                    .usage(
                        vk::BufferUsageFlags::VERTEX_BUFFER
                            | vk::BufferUsageFlags::INDEX_BUFFER
                            | vk::BufferUsageFlags::TRANSFER_DST,
                    )
                    .size(buffer_byte_size as u64)
                    .queue_family_indices(&queue_indices)
                    .sharing_mode(if queue_indices.len() == 1 {
                        vk::SharingMode::EXCLUSIVE
                    } else {
                        vk::SharingMode::CONCURRENT
                    }),
                None,
            )?
        };
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let mem_index = Device::memory_type_index(
            &config,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .ok_or(vk::Result::ERROR_UNKNOWN)?;
        let memory = unsafe {
            device.allocate_memory(
                &vk::MemoryAllocateInfo::builder()
                    .allocation_size(requirements.size)
                    .memory_type_index(mem_index),
                None,
            )?
        };
        unsafe { device.bind_buffer_memory(buffer, memory, 0)? };

        {
            let staging_buffer = Device::create_staging_buffer(device, config, staging_byte_size)?;
            Device::copy_buffer_data(
                device,
                &staging_buffer,
                config,
                command_pools,
                queues,
                buffer,
                0,
                &vertices,
            )?;
            Device::copy_buffer_data(
                device,
                &staging_buffer,
                config,
                command_pools,
                queues,
                buffer,
                vertex_byte_size,
                &indices,
            )?;
        }

        Ok(MeshData {
            memory,
            buffer,
            vertex_offset: 0,
            index_offset: vertex_byte_size,
            mesh_offsets,
        })
    }

    pub(super) fn bind_buffers(
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        data: &MeshData,
    ) {
        unsafe {
            device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &[data.buffer],
                &[data.vertex_offset as vk::DeviceSize],
            );
            device.cmd_bind_index_buffer(
                command_buffer,
                data.buffer,
                data.index_offset as vk::DeviceSize,
                vk::IndexType::UINT32,
            )
        }
    }

    pub(super) fn create_staging_buffer(
        device: &'a ash::Device,
        config: &PhysicalDeviceConfig,
        size: usize,
    ) -> VkResult<StagingBuffer<'a>> {
        let buffer = unsafe {
            device.create_buffer(
                &vk::BufferCreateInfo::builder()
                    .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                    .size(size as vk::DeviceSize)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .queue_family_indices(&[config.queue_families.transfer]),
                None,
            )?
        };
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let mem_index = Device::memory_type_index(
            &config,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .ok_or(vk::Result::ERROR_UNKNOWN)?;
        let memory = unsafe {
            device.allocate_memory(
                &vk::MemoryAllocateInfo::builder()
                    .allocation_size(requirements.size)
                    .memory_type_index(mem_index),
                None,
            )?
        };
        unsafe { device.bind_buffer_memory(buffer, memory, 0)? };
        let fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None)? };
        Ok(StagingBuffer {
            buffer,
            memory,
            device,
            fence,
        })
    }

    fn copy_buffer_data<T: Pod>(
        device: &ash::Device,
        staging_buffer: &StagingBuffer,
        config: &PhysicalDeviceConfig,
        command_pools: &CommandPools,
        queues: &Queues,
        dst: vk::Buffer,
        dst_offset: usize,
        src: &[T],
    ) -> VkResult<()> {
        let src = bytemuck::cast_slice::<T, u8>(src);
        unsafe {
            let mem = device.map_memory(
                staging_buffer.memory,
                0,
                src.len() as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            )?;
            copy_nonoverlapping(src.as_ptr(), mem as *mut u8, src.len());
            device.unmap_memory(staging_buffer.memory);
        };
        let command = Device::begin_single_time_command(
            device,
            config,
            command_pools,
            queues,
            CommandType::Transfer,
        )?;
        unsafe {
            device.cmd_copy_buffer(
                command.buffer,
                staging_buffer.buffer,
                dst,
                &[vk::BufferCopy {
                    src_offset: 0,
                    dst_offset: dst_offset as vk::DeviceSize,
                    size: src.len() as vk::DeviceSize,
                }],
            )
        }
        command.submit(device, Some(staging_buffer.fence))?;
        unsafe {
            device.wait_for_fences(&[staging_buffer.fence], true, u64::MAX)?;
            device.reset_fences(&[staging_buffer.fence])?;
        }
        Device::destory_command(device, command);
        Ok(())
    }

    pub(super) fn destory_mesh_data(device: &ash::Device, data: &mut MeshData) {
        unsafe {
            device.destroy_buffer(data.buffer, None);
            device.free_memory(data.memory, None);
        }
    }
}

impl<'a> Drop for StagingBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
            self.device.destroy_fence(self.fence, None);
        }
    }
}
