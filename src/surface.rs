use wgpu::{Instance, Surface, Queue, Device, SurfaceConfiguration};
use winit::window::Window;



/*
window: Oluşturulacak yüzeyin bağlı olduğu pencere.
instance: Vulkan örneği.
power_preference: Adaptör seçimi için güç tercihi.
compatible_surface: Yüzeyin uygunluğunu kontrol etmek için kullanılan yüzey.
force_fallback_adapter: Adaptör seçimi için yedek adaptör kullanımı.
features: Kullanılacak GPU özellikleri (örneğin, doku sıkıştırma).
limits: Cihazın sınırları.
label: Cihazın etiketi.
usage: Yüzeyin kullanımı.
format: Yüzeyin biçimi.
width: Yüzeyin genişliği.
height: Yüzeyin yüksekliği.
present_mode: Yüzeyin sunum modu.
alpha_mode: Yüzeyin alfa modu.
view_formats: Yüzeyin görünüm biçimleri.
 */
pub async fn create_surface(window: &Window,instance: &Instance) -> (Surface,Device,Queue,SurfaceConfiguration) {
    let surface = unsafe { instance.create_surface(window) }.unwrap();


    let adapter = instance
    .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    })
    .await
    .unwrap();



    let (device, queue) = adapter
    .request_device(
        &wgpu::DeviceDescriptor {
            features: adapter.features(), // features of the gpu that we want to use (ex: texture compression)
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            label: None,
        },
        None,
    )
    .await
    .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);


    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .filter(|f| f.describe().srgb)
        .next()
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: surface_caps.present_modes[1], // 0 is fifo (first in first out) 1 is mailbox (newest frame) 2 is immediate (tearing) 3 is vsync
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    (surface,device,queue,config)
}