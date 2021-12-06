use super::{CommandPools, Device, PhysicalDeviceConfig, Queues};
use ash::{prelude::VkResult, vk};

pub(super) enum CommandType {
    Graphics,
    Transfer,
    Compute,
}

pub(super) struct Command {
    pub(super) buffer: vk::CommandBuffer,
    pub(super) queue: vk::Queue,
    pool: vk::CommandPool,
}

impl Device {
    pub(super) fn begin_single_time_command(
        device: &ash::Device,
        config: &PhysicalDeviceConfig,
        command_pools: &CommandPools,
        queues: &Queues,
        command: CommandType,
    ) -> VkResult<Command> {
        let (queue, index, pool) = match command {
            CommandType::Graphics => (
                queues.graphics,
                config.queue_families.graphics,
                command_pools.graphics,
            ),
            CommandType::Compute => (
                queues.compute,
                config.queue_families.compute,
                command_pools.compute,
            ),
            CommandType::Transfer => (
                queues.transfer,
                config.queue_families.transfer,
                command_pools.transfer,
            ),
        };
        let buffer = unsafe {
            device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::builder()
                    .command_buffer_count(1)
                    .command_pool(pool)
                    .level(vk::CommandBufferLevel::PRIMARY),
            )?[0]
        };
        unsafe {
            device.begin_command_buffer(
                buffer,
                &vk::CommandBufferBeginInfo::builder()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
        }
        Ok(Command {
            buffer,
            pool,
            queue,
        })
    }

    pub(super) fn destory_command(device: &ash::Device, command: Command) {
        unsafe {
            device.free_command_buffers(command.pool, &[command.buffer]);
        }
    }
}

impl Command {
    pub fn submit(&self, device: &ash::Device, fence: Option<vk::Fence>) -> VkResult<()> {
        unsafe {
            device.end_command_buffer(self.buffer)?;
            device.queue_submit(
                self.queue,
                &[vk::SubmitInfo::builder()
                    .command_buffers(&[self.buffer])
                    .build()],
                fence.unwrap_or(vk::Fence::null()),
            )?;
        };
        Ok(())
    }
}
