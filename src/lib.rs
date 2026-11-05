use std::sync::Arc;

use wgpu::{DeviceDescriptor, ExperimentalFeatures, InstanceDescriptor, RequestAdapterOptions, SurfaceConfiguration};
use winit::{
    application::ApplicationHandler, event::*, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::{event_loop::{EventLoop, EventLoopProxy}, platform::web::EventLoopExtWebSys};

// State of the Game
pub struct State {
    surface: wgpu::Surface<'static>, // Die Oberfläche im Fenster auf die gemalt wird
    device: wgpu::Device, // Die GPU
    queue: wgpu::Queue, // Die Queue für die Befehle an die GPU
    config: wgpu::SurfaceConfiguration,
    is_surface_configed: bool,
    window: Arc<Window>, // Atomicallly Refrence Counted
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // Instance is the primary handle for the GPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            }).await?;

        println!("Adapter created. \n{:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") { // Im Web weniger Features
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(), // Bevorzuge allocation strategien
                trace: wgpu::Trace::Off,
            }).await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // To write on screen
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        Ok(
            Self {
                surface,
                device,
                queue,
                config,
                is_surface_configed: false,
                window,
            }
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configed = true;
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        // Wenn nicht configured nicht renderbar
        if !self.is_surface_configed {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,

            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // Skip this frame
                return Ok(());
            }

            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // You could recreate the devices and all resources
                // created with it here, but we'll just bail
                anyhow::bail!("Lost device");
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        // Extra Block damit encoder geropt wird
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.5,
                            b: 0.5,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }

    pub fn update() {
        
    }
    
    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<EventLoopProxy<State>>,
    state: Option<State>,
}

impl App {
    pub fn new( #[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
         #[cfg(target_arch = "wasm32")]
         let proxy = Some(event_loop.create_proxy());
         Self {
             state: None,
             #[cfg(target_arch = "wasm32")]
             proxy,
         }
    }
}

// Zuzang zu Mausbewegungn, life cycle events
impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("DAS LÄUFT EINFACH AUF DER GPU WASS?");
        
        #[cfg(target_arch = "wasm32")]
        {
            // Machen habe ich keine Lust drauf
            todo!()
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        println!("Window created!");
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Nicht im web, pollster to await window creation
            self.state = Some(pollster::block_on(State::new(window)).unwrap());        }
        }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: State) {
        #[cfg(target_arch = "wasm32")]
        {
            todo!()
        }
        self.state = Some(event);
    }

    // Process Keyboard Inputs, mouse movements
    // or if window wants to draw, resize
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    )
    {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                State::update(); // das hier ist denke ich falsch
                match state.render() {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                .. // Das wusste ich nicht das das geht aber das nimmt die Default Implementaionen
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")] 
    {
        todo!()
    }
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}