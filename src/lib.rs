use std::sync::Arc;

use winit::{
    application::ApplicationHandler, event::*, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::{event_loop::{EventLoop, EventLoopProxy}, platform::web::EventLoopExtWebSys};

// State of the Game
pub struct State {
    window: Arc<Window>, // Atomicallly Refrence Counted
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        Ok(
            Self { window, }
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // TODO
    }

    pub fn render(&mut self) {
        self.window.request_redraw();

        // TODO
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
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            // Machen habe ich keine Lust drauf
            todo!()
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Nicht im web, pollster to await window creation
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
            eprintln!("Getting here");
        }
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
            WindowEvent::RedrawRequested => {
                state.render();
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                .. // Das wusste ich nicht das das geht aber das nimmt die Default Implementaionen
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
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