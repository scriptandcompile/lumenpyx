use glium;
use glium::glutin::surface::WindowSurface;
use glium::implement_vertex;
use glium::Surface;
pub use winit;
use winit::event_loop::EventLoop;
pub mod primitives;
mod shaders;
use shaders::*;
mod drawable_object;
pub use drawable_object::*;
use rustc_hash::FxHashMap;

pub(crate) const WINDOW_VIRTUAL_SIZE: [u32; 2] = [128, 128];
pub(crate) const DEFAULT_BEHAVIOR: glium::uniforms::SamplerBehavior =
    glium::uniforms::SamplerBehavior {
        minify_filter: glium::uniforms::MinifySamplerFilter::Nearest,
        magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
        max_anisotropy: 1,
        wrap_function: (
            glium::uniforms::SamplerWrapFunction::Mirror,
            glium::uniforms::SamplerWrapFunction::Mirror,
            glium::uniforms::SamplerWrapFunction::Mirror,
        ),
        depth_texture_comparison: None,
    };

pub struct LumenpyxProgram {
    pub window: winit::window::Window,
    pub display: glium::Display<WindowSurface>,
    pub indices: glium::index::NoIndices,
    pub lighting_shader: glium::Program,
    pub reflection_shader: glium::Program,
    pub upscale_shader: glium::Program,
    other_shaders: FxHashMap<String, glium::Program>,
}

impl LumenpyxProgram {
    pub fn new() -> (LumenpyxProgram, EventLoop<()>) {
        let (event_loop, window, display, indices) = setup_program();
        let lighting_shader = glium::Program::from_source(
            &display,
            shaders::LIGHTING_VERTEX_SHADER_SRC,
            shaders::LIGHTING_FRAGMENT_SHADER_SRC,
            None,
        )
        .unwrap();

        let reflection_shader = glium::Program::from_source(
            &display,
            shaders::REFLECTION_VERTEX_SHADER_SRC,
            shaders::REFLECTION_FRAGMENT_SHADER_SRC,
            None,
        )
        .unwrap();

        let upscale_shader = glium::Program::from_source(
            &display,
            shaders::UPSCALE_VERTEX_SHADER_SRC,
            shaders::UPSCALE_FRAGMENT_SHADER_SRC,
            None,
        )
        .unwrap();

        (
            LumenpyxProgram {
                window,
                display,
                indices,
                lighting_shader,
                reflection_shader,
                upscale_shader,
                other_shaders: FxHashMap::default(),
            },
            event_loop,
        )
    }

    pub fn add_shader(&mut self, program: glium::Program, name: &str) {
        self.other_shaders.insert(name.to_string(), program);
    }

    pub fn get_shader(&self, name: &str) -> Option<&glium::Program> {
        self.other_shaders.get(name)
    }
}

#[derive(Copy, Clone)]
pub struct Transform {
    matrix: [[f32; 4]; 4],
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.matrix[3][0] = x;
        self.matrix[3][1] = y;
        self.matrix[3][2] = z;
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.matrix[0][0] = x;
        self.matrix[1][1] = y;
        self.matrix[2][2] = z;
    }

    pub fn set_x(&mut self, x: f32) {
        self.matrix[3][0] = x;
    }

    pub fn set_y(&mut self, y: f32) {
        self.matrix[3][1] = y;
    }

    pub fn set_z(&mut self, z: f32) {
        self.matrix[3][2] = z;
    }
}

#[derive(Copy, Clone)]
pub struct Light {
    position: [f32; 3],
    color: [f32; 3],
    intensity: f32,
    falloff: f32,
}

impl Light {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32, falloff: f32) -> Light {
        Light {
            position,
            color,
            intensity,
            falloff,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = [x, y, z];
    }

    pub fn get_position(&self) -> [f32; 3] {
        self.position
    }

    /// Set the color of the light in 0.0 - 1.0 range
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.color = [r, g, b];
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }

    pub fn set_falloff(&mut self, falloff: f32) {
        self.falloff = falloff;
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub fn setup_program() -> (
    EventLoop<()>,
    winit::window::Window,
    glium::Display<WindowSurface>,
    glium::index::NoIndices,
) {
    // this is just a wrapper for the setup_window function for now
    let (event_loop, display, window, indices) = setup_window();

    (event_loop, window, display, indices)
}

fn load_image(path: &str) -> glium::texture::RawImage2d<u8> {
    let img = image::open(path).unwrap();
    img.flipv();
    let path = format!("{}", path);
    let image = image::load(
        std::io::Cursor::new(std::fs::read(path).unwrap()),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image, image_dimensions);
    image
}

fn setup_window() -> (
    EventLoop<()>,
    glium::Display<WindowSurface>,
    winit::window::Window,
    glium::index::NoIndices,
) {
    // 1. The **winit::EventLoop** for handling events.
    let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
    // 2. Create a glutin context and glium Display
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    (event_loop, display, window, indices)
}

pub fn draw_all(
    /*display: &glium::Display<WindowSurface>,
    indices: &glium::index::NoIndices,*/
    lights: Vec<&Light>,
    drawables: Vec<&impl Drawable>,
    program: &mut LumenpyxProgram,
) {
    for drawable in &drawables {
        drawable.try_load_shaders(program);
    }
    /*
    STEP 1:
        render every albedo to a texture
        render every height to a texture
        render every roughness to a texture
    STEP 2:
        take the textures and feed it into a lighting shader
        we do this for every light and then blend the results together
    STEP 3:
        take the result and feed it into a reflection shader
        it uses screen space reflections and lerps between the reflection and the original image based on the roughness
    STEP 4:
        upscale the result to the screen size
    */
    let display = &program.display;
    let indices = &program.indices;

    let albedo_texture = glium::texture::Texture2d::empty_with_format(
        display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        WINDOW_VIRTUAL_SIZE[0],
        WINDOW_VIRTUAL_SIZE[1],
    )
    .unwrap();

    let height_texture = glium::texture::Texture2d::empty_with_format(
        display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        WINDOW_VIRTUAL_SIZE[0],
        WINDOW_VIRTUAL_SIZE[1],
    )
    .unwrap();

    let normal_texture = glium::texture::Texture2d::empty_with_format(
        display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        WINDOW_VIRTUAL_SIZE[0],
        WINDOW_VIRTUAL_SIZE[1],
    )
    .unwrap();

    let roughness_texture = glium::texture::Texture2d::empty_with_format(
        display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        WINDOW_VIRTUAL_SIZE[0],
        WINDOW_VIRTUAL_SIZE[1],
    )
    .unwrap();

    {
        let mut albedo_framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(display, &albedo_texture).unwrap();
        albedo_framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);

        let mut height_framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(display, &height_texture).unwrap();
        height_framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);

        let mut roughness_framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(display, &roughness_texture).unwrap();
        roughness_framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);

        let mut normal_framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(display, &normal_texture).unwrap();
        normal_framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);

        for drawable in &drawables {
            drawable.draw(
                program,
                &mut albedo_framebuffer,
                &mut height_framebuffer,
                &mut roughness_framebuffer,
                &mut normal_framebuffer,
            )
        }
    }

    let lit_texture = glium::texture::Texture2d::empty_with_format(
        display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        WINDOW_VIRTUAL_SIZE[0],
        WINDOW_VIRTUAL_SIZE[1],
    )
    .expect("Failed to create lit frame buffer");

    {
        let albedo = glium::uniforms::Sampler(&albedo_texture, DEFAULT_BEHAVIOR);
        let height_sampler = glium::uniforms::Sampler(&height_texture, DEFAULT_BEHAVIOR);

        let mut lit_framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(display, &lit_texture).unwrap();
        lit_framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);

        for light in lights {
            draw_lighting(
                albedo,
                height_sampler,
                light,
                &program,
                &mut lit_framebuffer,
            );
        }
    }

    let reflected_texture = glium::texture::Texture2d::empty_with_format(
        display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        WINDOW_VIRTUAL_SIZE[0],
        WINDOW_VIRTUAL_SIZE[1],
    )
    .expect("Failed to create reflected frame buffer");

    {
        let roughness = glium::uniforms::Sampler(&roughness_texture, DEFAULT_BEHAVIOR);
        let height = glium::uniforms::Sampler(&height_texture, DEFAULT_BEHAVIOR);
        let normal = glium::uniforms::Sampler(&normal_texture, DEFAULT_BEHAVIOR);
        let lit_sampler = glium::uniforms::Sampler(&lit_texture, DEFAULT_BEHAVIOR);

        let mut reflected_framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(display, &reflected_texture).unwrap();

        draw_reflections(
            lit_sampler,
            height,
            roughness,
            normal,
            &mut reflected_framebuffer,
            &program,
        );
    }

    {
        let finished_texture = glium::uniforms::Sampler(&lit_texture, DEFAULT_BEHAVIOR);
        draw_upscale(finished_texture, &program);
    }
}
