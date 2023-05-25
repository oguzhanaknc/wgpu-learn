mod vertex; // vertex class ını ekledim
mod render_pipeline; // render pipeline oluşturma fonksiyonunu ekledim 
mod surface; // surface oluşturma fonksiyonunu ekledim
mod buffer; // buffer oluşturma fonksiyonunu ekledim
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};




const VERTICES: &[vertex::Vertex] = &[
    vertex::Vertex { position: [-0.5, -0.5, 0.5], color: [1.0, 0.0, 0.0] },
    vertex::Vertex { position: [0.5, -0.5, 0.5], color: [0.0, 1.0, 0.0] },
    vertex::Vertex { position: [0.5, 0.5, 0.5], color: [0.0, 0.0, 1.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2
];


// state sınıfını tanımlıyorum
struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>, // size of the window in pixels
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer, // vertex buffer is a buffer that contains the vertices of the triangle
    index_buffer : wgpu::Buffer,
    num_indices: u32,
}

// state sınıfı için fonksiyonlar tanımlıyorum
impl State {
    
    // state sınıfının oluşturucu fonksiyonu
    async fn new(window: Window) -> Self { // Self is the type of the struct
        let size = window.inner_size(); // size of the window in pixels
 
        /*
                1-) Window oluşturulur. 
                2-) Instance oluşturulur.
                3-) Surface oluşturulur.
                4-) Device oluşturulur.
                5-) Queue oluşturulur.
                6-) SurfaceConfiguration oluşturulur.
                7-) RenderPipeline oluşturulur.
                8-) Vertex Buffer oluşturulur.
                9-) Index Buffer oluşturulur.
        
         */


        // The instance is  a handle to our gpu
        // backends: Vulkan, Metal, DX12, Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            dx12_shader_compiler: Default::default(),
        });
      
        let (surface,device,queue,config)
        = surface::create_surface(&window, &instance).await;
          

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline = render_pipeline::create_render_pipeline_default(&device, &shader,config.format);
       
        let vertex_buffer = buffer::create_buffer(&device, "Vertex Buffer", &VERTICES, wgpu::BufferUsages::VERTEX);
        let index_buffer = buffer::create_buffer(&device, "Index Buffer", &INDICES, wgpu::BufferUsages::INDEX);



        let num_indices = INDICES.len() as u32;
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
    fn update(&mut self) {}
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // create a command encoder which we can use to submit commands to the gpu
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..)); // 3.
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 2.
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    // Window setup...
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // State setup...
    let mut state = State::new(window).await;

    // Event loop...
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    // UPDATED!
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}



 