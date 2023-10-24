use femtovg::{
    renderer::OpenGl,
    Align,
    Baseline,
    Color,
    FontId,
    ImageFlags,
    ImageId,
    Paint,
    Path,
    Canvas,
    renderer::Renderer,
};
use image;

#[cfg(target_os="macos")]
fn get_renderer() -> OpenGl {
    use glutin::dpi::PhysicalSize;
    use glutin::event_loop::EventLoop;
    use glutin::{
        Context, ContextBuilder, ContextCurrentState, CreationError, GlProfile, GlRequest, NotCurrent,
    };

    let cb = ContextBuilder::new().with_gl_profile(GlProfile::Core)
                                  .with_gl(GlRequest::Latest);
    //let size     = PhysicalSize::new(1920., 1080.);
    let el       = EventLoop::new();
    let size_one = PhysicalSize::new(1, 1);
    let ctx      = cb.build_headless(&el, size_one).unwrap();
    let ctx      = unsafe { ctx.make_current().unwrap() };
    let renderer = unsafe { OpenGl::new_from_function(|s| ctx.get_proc_address(s) as *const _) }
        .expect("Cannot create renderer");
    return renderer
}

#[cfg(not(target_os="macos"))]
fn get_renderer() -> Opengl {
    use glutin::config::{ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
    use glutin::display::GetGlDisplay;
    use glutin::context::{ContextApi, ContextAttributesBuilder};
    use glutin::prelude::*;
    use glutin::api::egl::device::Device;
    use glutin::api::egl::display::Display;

    let devices = Device::query_devices().expect("Failed to query devices").collect::<Vec<_>>();
    for (index, device) in devices.iter().enumerate() {
        println!(
            "Device {}: Name: {} Vendor: {}",
            index,
            device.name().unwrap_or("UNKNOWN"),
            device.vendor().unwrap_or("UNKNOWN")
            );
    }

    let device = devices.first().expect("No available devices");
    let display = unsafe { Display::with_device(device, None) }.expect("Failed to create display");
    let template = ConfigTemplateBuilder::default()
        .with_alpha_size(8)
        .with_surface_type(ConfigSurfaceTypes::empty())
        .build();

    let config = unsafe { display.find_configs(template) }
        .unwrap()
        .reduce(|config, acc| {
                if config.num_samples() > acc.num_samples() {
                    config
                } else {
                    acc
                }
        })
        .expect("No available configs");
    println!("Picked a config with {} samples", config.num_samples());

    let context_attributes = ContextAttributesBuilder::new().build(None);
    let fallback_context_attributes =
        ContextAttributesBuilder::new().with_context_api(ContextApi::Gles(None)).build(None);
    let not_current = unsafe {
        display.create_context(&config, &context_attributes).unwrap_or_else(|_| {
            display
                .create_context(&config, &fallback_context_attributes)
                .expect("failed to create context")
        })
    };
    let _context = not_current.make_current_surfaceless().unwrap();
    let renderer = unsafe { OpenGl::new_from_function_cstr(|s| display.get_proc_address(s) as *const _) }
        .expect("Cannot create renderer");
    return renderer
}

fn main() {
    let renderer = get_renderer();
    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

    let (width, height) = (640, 480);
    canvas.set_size(width, height, 1.0);
    canvas.clear_rect(0, 0, 1920, 1080, Color::rgbf(0.9, 0.9, 0.9));
    canvas.flush();

    let screenshot = canvas.screenshot().unwrap();
    println!("{:?}", screenshot);
    let buf = screenshot.into_contiguous_buf();
    let mut imgbuf = image::RgbaImage::new(width, height);
    
    for (p1, p0) in imgbuf.pixels_mut().zip(buf.0) {
        *p1 = image::Rgba([p0.r, p0.g, p0.b, p0.a]);
    }

    imgbuf.save("test.png").unwrap();
}
