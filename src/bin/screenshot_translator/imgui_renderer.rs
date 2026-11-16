use std::error::Error;
use std::ffi::{c_int, c_void};
use std::fmt::{self, Display, Formatter};
use std::mem::offset_of;

use image::RgbaImage;
use imgui::internal::RawWrapper as _;
use sdl3::pixels::PixelFormat;
use sdl3::rect::Rect;
use sdl3::render::{
    BlendMode, Canvas, ClippingRect, RenderTarget, ScaleMode, Texture, TextureCreator,
    TextureValueError, UpdateTextureError,
};
use sdl3::sys;
use sdl3::sys::pixels::SDL_FColor;

pub struct Renderer<'a, Target> {
    texture_creator: &'a TextureCreator<Target>,
    texture_map: imgui::Textures<Texture<'a>>,
    image_texture: Option<imgui::TextureId>,
    image_size: Option<(u32, u32)>,
    color_buffer: Vec<SDL_FColor>,
}

impl<'a, Target> Renderer<'a, Target> {
    pub fn new(
        texture_creator: &'a TextureCreator<Target>,
        context: &mut imgui::Context,
    ) -> Result<Self, RendererError> {
        let mut texture_map = imgui::Textures::new();
        prepare_font_atlas(texture_creator, context, &mut texture_map)?;
        context.set_renderer_name(Some(format!(
            "screenshot-translator-sdl3-renderer {}",
            env!("CARGO_PKG_VERSION")
        )));
        context
            .io_mut()
            .backend_flags
            .insert(imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET);
        Ok(Self {
            texture_creator,
            texture_map,
            image_texture: None,
            image_size: None,
            color_buffer: Vec::new(),
        })
    }

    pub fn set_image(&mut self, image: &RgbaImage) -> Result<imgui::TextureId, RendererError> {
        let (width, height) = image.dimensions();
        let texture_identifier = match self.image_texture {
            Some(identifier) if self.image_size == Some((width, height)) => {
                let texture = self
                    .texture_map
                    .get_mut(identifier)
                    .ok_or(RendererError::MissingTexture)?;
                texture.update(None, image.as_raw(), width as usize * 4)?;
                identifier
            }
            _ => {
                let mut texture = self.texture_creator.create_texture_streaming(
                    PixelFormat::RGBA32,
                    width,
                    height,
                )?;
                texture.update(None, image.as_raw(), width as usize * 4)?;
                texture.set_blend_mode(BlendMode::Blend);
                texture.set_scale_mode(ScaleMode::Linear);
                let identifier = self.texture_map.insert(texture);
                if let Some(previous_identifier) = self.image_texture.replace(identifier) {
                    self.texture_map.remove(previous_identifier);
                }
                self.image_size = Some((width, height));
                identifier
            }
        };
        self.image_texture = Some(texture_identifier);
        Ok(texture_identifier)
    }

    pub fn render(
        &mut self,
        draw_data: &imgui::DrawData,
        canvas: &mut Canvas<impl RenderTarget>,
    ) -> Result<(), RendererError> {
        let viewport = canvas.viewport();
        let clipping_rectangle = canvas.clip_rect();
        set_up_render_state(canvas);
        let framebuffer_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let framebuffer_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if framebuffer_width == 0.0 || framebuffer_height == 0.0 {
            canvas.set_viewport(viewport);
            canvas.set_clip_rect(clipping_rectangle);
            return Ok(());
        }
        for draw_list in draw_data.draw_lists() {
            for command in draw_list.commands() {
                match command {
                    imgui::DrawCmd::Elements { count, cmd_params } => {
                        render_elements(
                            &self.texture_map,
                            &mut self.color_buffer,
                            canvas,
                            draw_list.vtx_buffer(),
                            draw_list.idx_buffer(),
                            count,
                            &cmd_params,
                            &draw_data.display_pos,
                            &draw_data.framebuffer_scale,
                            (framebuffer_width, framebuffer_height),
                        )?;
                    }
                    imgui::DrawCmd::ResetRenderState => set_up_render_state(canvas),
                    imgui::DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd);
                    },
                }
            }
        }
        canvas.set_viewport(viewport);
        canvas.set_clip_rect(clipping_rectangle);
        Ok(())
    }
}

fn prepare_font_atlas<'a, Target>(
    texture_creator: &'a TextureCreator<Target>,
    context: &mut imgui::Context,
    texture_map: &mut imgui::Textures<Texture<'a>>,
) -> Result<(), RendererError> {
    let font_atlas = context.fonts().build_rgba32_texture();
    let mut font_texture = texture_creator.create_texture_static(
        PixelFormat::RGBA32,
        font_atlas.width,
        font_atlas.height,
    )?;
    font_texture.update(None, font_atlas.data, font_atlas.width as usize * 4)?;
    font_texture.set_blend_mode(BlendMode::Blend);
    font_texture.set_scale_mode(ScaleMode::Linear);
    let texture_identifier = texture_map.insert(font_texture);
    context.fonts().tex_id = texture_identifier;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_elements<'a>(
    texture_map: &imgui::Textures<Texture<'a>>,
    color_buffer: &mut Vec<SDL_FColor>,
    canvas: &mut Canvas<impl RenderTarget>,
    vertex_buffer: &[imgui::DrawVert],
    index_buffer: &[imgui::DrawIdx],
    element_count: usize,
    element_params: &imgui::DrawCmdParams,
    display_position: &[f32; 2],
    framebuffer_scale: &[f32; 2],
    framebuffer_size: (f32, f32),
) -> Result<(), RendererError> {
    let clip_minimum = (
        (element_params.clip_rect[0] - display_position[0]) * framebuffer_scale[0],
        (element_params.clip_rect[1] - display_position[1]) * framebuffer_scale[1],
    );
    let clip_maximum = (
        (element_params.clip_rect[2] - display_position[0]) * framebuffer_scale[0],
        (element_params.clip_rect[3] - display_position[1]) * framebuffer_scale[1],
    );
    if clip_minimum.0 >= framebuffer_size.0
        || clip_minimum.1 >= framebuffer_size.1
        || clip_maximum.0 < 0.0
        || clip_maximum.1 < 0.0
    {
        return Ok(());
    }
    let clipping_rectangle = Rect::new(
        clip_minimum.0 as i32,
        clip_minimum.1 as i32,
        (clip_maximum.0 - clip_minimum.0) as u32,
        (clip_maximum.1 - clip_minimum.1) as u32,
    );
    canvas.set_clip_rect(clipping_rectangle);
    let texture = texture_map.get(element_params.texture_id);
    render_raw_geometry(
        canvas,
        color_buffer,
        texture,
        &vertex_buffer[element_params.vtx_offset..],
        &index_buffer[element_params.idx_offset..element_params.idx_offset + element_count],
    )
}

fn render_raw_geometry(
    canvas: &mut Canvas<impl RenderTarget>,
    color_buffer: &mut Vec<SDL_FColor>,
    texture: Option<&Texture>,
    vertices: &[imgui::DrawVert],
    indices: &[imgui::DrawIdx],
) -> Result<(), RendererError> {
    let vertex_stride = size_of::<imgui::DrawVert>() as c_int;
    color_buffer.clear();
    color_buffer.extend(vertices.iter().map(|vertex| SDL_FColor {
        r: vertex.col[0] as f32 / 255.0,
        g: vertex.col[1] as f32 / 255.0,
        b: vertex.col[2] as f32 / 255.0,
        a: vertex.col[3] as f32 / 255.0,
    }));
    let renderer = canvas.raw();
    let texture = texture.map_or(std::ptr::null_mut(), Texture::raw);
    let positions = unsafe {
        vertices
            .as_ptr()
            .byte_add(offset_of!(imgui::DrawVert, pos))
            .cast::<f32>()
    };
    let texture_coordinates = unsafe {
        vertices
            .as_ptr()
            .byte_add(offset_of!(imgui::DrawVert, uv))
            .cast::<f32>()
    };
    let success = unsafe {
        sys::render::SDL_RenderGeometryRaw(
            renderer,
            texture,
            positions,
            vertex_stride,
            color_buffer.as_ptr(),
            size_of::<SDL_FColor>() as c_int,
            texture_coordinates,
            vertex_stride,
            vertices.len() as c_int,
            indices.as_ptr().cast::<c_void>(),
            indices.len() as c_int,
            size_of::<imgui::DrawIdx>() as c_int,
        )
    };
    if success {
        Ok(())
    } else {
        Err(RendererError::Sdl(sdl3::get_error()))
    }
}

fn set_up_render_state(canvas: &mut Canvas<impl RenderTarget>) {
    canvas.set_clip_rect(ClippingRect::None);
    canvas.set_viewport(None);
}

#[derive(Debug)]
pub enum RendererError {
    UpdateTexture(UpdateTextureError),
    TextureValue(TextureValueError),
    Sdl(sdl3::Error),
    MissingTexture,
}

impl Display for RendererError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpdateTexture(error) => write!(formatter, "{error}"),
            Self::TextureValue(error) => write!(formatter, "{error}"),
            Self::Sdl(error) => write!(formatter, "{error}"),
            Self::MissingTexture => write!(formatter, "image texture is missing"),
        }
    }
}

impl Error for RendererError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UpdateTexture(error) => Some(error),
            Self::TextureValue(error) => Some(error),
            Self::Sdl(error) => Some(error),
            Self::MissingTexture => None,
        }
    }
}

impl From<UpdateTextureError> for RendererError {
    fn from(error: UpdateTextureError) -> Self {
        Self::UpdateTexture(error)
    }
}

impl From<TextureValueError> for RendererError {
    fn from(error: TextureValueError) -> Self {
        Self::TextureValue(error)
    }
}

impl From<sdl3::Error> for RendererError {
    fn from(error: sdl3::Error) -> Self {
        Self::Sdl(error)
    }
}
