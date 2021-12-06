use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    renderer,
    scene::{Scene, SceneBuilder},
    utils::StaticResult,
};

const DEFAULT_WINDOW_HEIGHT: u32 = 728;
const DEFAULT_WINDOW_WIDTH: u32 = 1024;
const DEFAULT_APPLICATION_TITLE: &'static str = "RustGamephysics";
const DEFAULT_RENDERER_BACKEND: renderer::Backend = renderer::Backend::Vulkan;

pub struct ApplicationBuilder {
    title: &'static str,
    extent: (u32, u32),
    backend: renderer::Backend,
    scene_builder: Option<SceneBuilder>,
}

pub struct Application {
    window: Window,
    event_loop: EventLoop<()>,
    renderer: Box<dyn renderer::Renderer>,
    scene: Scene,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        Self {
            title: DEFAULT_APPLICATION_TITLE,
            extent: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            backend: DEFAULT_RENDERER_BACKEND,
            scene_builder: None,
        }
    }

    pub fn with_title(self, title: &'static str) -> Self {
        Self { title, ..self }
    }

    pub fn with_window_size(self, width: u32, height: u32) -> Self {
        Self {
            extent: (width, height),
            ..self
        }
    }

    pub fn with_backend(self, backend: renderer::Backend) -> Self {
        Self { backend, ..self }
    }

    pub fn with_scene(self, scene: SceneBuilder) -> Self {
        Self {
            scene_builder: Some(scene),
            ..self
        }
    }

    pub fn build(self) -> StaticResult<Application> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(self.title)
            .with_inner_size(PhysicalSize::new(self.extent.0, self.extent.1))
            .with_resizable(false)
            .build(&event_loop)?;
        let scene_builder = self.scene_builder.ok_or(format!("Scene not provided"))?;
        let renderer = renderer::create(self.backend, &window, &scene_builder.meshes)?;
        let scene = scene_builder.build(
            60.0,
            (self.extent.0 as f32) / (self.extent.1 as f32),
            0.001,
            10000.0,
        )?;
        Ok(Application {
            window,
            event_loop,
            renderer,
            scene,
        })
    }
}

impl Application {
    pub fn run(self) {
        let Application {
            window,
            event_loop,
            mut renderer,
            scene,
        } = self;
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    renderer.begin_frame(&scene.camera).unwrap();
                    for object in &scene.objects {
                        renderer.draw(object.mesh, &object.world);
                    }
                    renderer.end_frame().unwrap();
                }
                Event::LoopDestroyed => {}
                _ => {}
            }
        });
    }
}
