/*!
Framebuffers allow you to customize the color, depth and stencil buffers you will draw on.

In order to draw on a texture, use a `SimpleFrameBuffer`. This framebuffer is compatible with
shaders that write to `gl_FragColor`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# let texture: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);
// framebuffer.draw(...);    // draws over `texture`
```

If, however, your shader wants to write to multiple color buffers at once, you must use
a `MultiOutputFrameBuffer`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# let texture1: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
# let texture2: glium::texture::Texture2d = unsafe { ::std::mem::uninitialized() };
let output = &[ ("output1", &texture1), ("output2", &texture2) ];
let framebuffer = glium::framebuffer::MultiOutputFrameBuffer::new(&display, output);
// framebuffer.draw(...);

// example shader:
// 
//     out vec4 output1;
//     out vec4 output2;
//
//     void main() {
//         output1 = vec4(0.0, 0.0, 0.5, 1.0);
//         output2 = vec4(1.0, 0.7, 1.0, 1.0);
//     }
```

**Note**: depth-stencil attachments are not yet implemented.

*/
use std::marker::PhantomData;
use std::rc::Rc;

use texture::Texture;
use texture::Texture2d;
use texture::{Texture1dMipmap, SrgbTexture1dMipmap, DepthTexture1dMipmap, StencilTexture1dMipmap, DepthStencilTexture1dMipmap};
use texture::{Texture2dMipmap, SrgbTexture2dMipmap, DepthTexture2dMipmap, StencilTexture2dMipmap, DepthStencilTexture2dMipmap};
use texture::{Texture2dMultisampleMipmap, SrgbTexture2dMultisampleMipmap, DepthTexture2dMultisampleMipmap, StencilTexture2dMultisampleMipmap, DepthStencilTexture2dMultisampleMipmap};
use texture::{Texture3dMipmap, SrgbTexture3dMipmap, DepthTexture3dMipmap, StencilTexture3dMipmap, DepthStencilTexture3dMipmap};
use texture::{Texture1dArrayMipmap, SrgbTexture1dArrayMipmap, DepthTexture1dArrayMipmap, StencilTexture1dArrayMipmap, DepthStencilTexture1dArrayMipmap};
use texture::{Texture2dArrayMipmap, SrgbTexture2dArrayMipmap, DepthTexture2dArrayMipmap, StencilTexture2dArrayMipmap, DepthStencilTexture2dArrayMipmap};
use texture::{Texture2dMultisampleArrayMipmap, SrgbTexture2dMultisampleArrayMipmap, DepthTexture2dMultisampleArrayMipmap, StencilTexture2dMultisampleArrayMipmap, DepthStencilTexture2dMultisampleArrayMipmap};

use backend::Facade;
use context::Context;

use fbo::FramebufferAttachments;
use FboAttachments;
use Rect;
use BlitTarget;
use ToGlEnum;
use ops;
use uniforms;

use {Program, Surface, GlObject};
use DrawError;

use {fbo, gl};

/// A framebuffer which has only one color attachment.
pub struct SimpleFrameBuffer<'a> {
    context: Rc<Context>,
    attachments: FramebufferAttachments,
    marker: PhantomData<&'a ()>,
    dimensions: (u32, u32),
    depth_buffer_bits: Option<u16>,
    stencil_buffer_bits: Option<u16>,
}

impl<'a> SimpleFrameBuffer<'a> {
    /// Creates a `SimpleFrameBuffer` with a single color attachment and no depth
    /// nor stencil buffer.
    pub fn new<F, C>(facade: &F, color: &'a C) -> SimpleFrameBuffer<'a>
                  where C: ToColorAttachment, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None, None, None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a depth
    /// buffer, but no stencil buffer.
    pub fn with_depth_buffer<F, C, D>(facade: &F, color: &'a C, depth: &'a D)
                                      -> SimpleFrameBuffer<'a>
                                      where C: ToColorAttachment, D: ToDepthAttachment, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(),
                                    Some(depth.to_depth_attachment()), None, None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment, a depth
    /// buffer, and a stencil buffer.
    pub fn with_depth_and_stencil_buffer<F, C, D, S>(facade: &F, color: &'a C, depth: &'a D,
                                                     stencil: &'a S) -> SimpleFrameBuffer<'a>
                                                     where C: ToColorAttachment,
                                                           D: ToDepthAttachment,
                                                           S: ToStencilAttachment, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(),
                                    Some(depth.to_depth_attachment()),
                                    Some(stencil.to_stencil_attachment()), None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a stencil
    /// buffer, but no depth buffer.
    pub fn with_stencil_buffer<F, C, S>(facade: &F, color: &'a C, stencil: &'a S)
                                        -> SimpleFrameBuffer<'a>
                                        where C: ToColorAttachment, S: ToStencilAttachment,
                                              F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None,
                                    Some(stencil.to_stencil_attachment()), None)
    }

    /// Creates a `SimpleFrameBuffer` with a single color attachment and a depth-stencil buffer.
    pub fn with_depth_stencil_buffer<F, C, D>(facade: &F, color: &'a C, depthstencil: &'a D)
                                              -> SimpleFrameBuffer<'a>
                                              where C: ToColorAttachment,
                                                    D: ToDepthStencilAttachment, F: Facade
    {
        SimpleFrameBuffer::new_impl(facade, color.to_color_attachment(), None, None,
                                    Some(depthstencil.to_depth_stencil_attachment()))
    }


    fn new_impl<F>(facade: &F, color: ColorAttachment, depth: Option<DepthAttachment>,
                   stencil: Option<StencilAttachment>, depthstencil: Option<DepthStencilAttachment>)
                   -> SimpleFrameBuffer<'a> where F: Facade
    {
        // TODO: remove this
        if depthstencil.is_some() {
            unimplemented!();
        }

        let (dimensions, color_attachment) = match color {
            ColorAttachment::Texture2d(tex) => {
                let dimensions = (tex.get_texture().get_width(), tex.get_texture().get_height().unwrap());
                let id = fbo::Attachment::Texture { id: tex.get_texture().get_id(), bind_point: gl::TEXTURE_2D, level: 0, layer: 0 };
                (dimensions, id)
            },

            ColorAttachment::Texture2dMultisample(tex) => {
                let dimensions = (tex.get_texture().get_width(), tex.get_texture().get_height().unwrap());
                let id = fbo::Attachment::Texture { id: tex.get_texture().get_id(), bind_point: gl::TEXTURE_2D_MULTISAMPLE, level: 0, layer: 0 };
                (dimensions, id)
            },

            ColorAttachment::RenderBuffer(buffer) => {
                let dimensions = buffer.get_dimensions();
                let id = fbo::Attachment::RenderBuffer(buffer.get_id());
                (dimensions, id)
            },

            _ => unimplemented!()
        };

        let (depth, depth_bits) = if let Some(depth) = depth {
            match depth {
                DepthAttachment::Texture2d(tex) => {
                    if (tex.get_texture().get_width(), tex.get_texture().get_height().unwrap()) != dimensions {
                        panic!("The depth attachment must have the same dimensions \
                                as the color attachment");
                    }

                    (Some(fbo::Attachment::Texture { id: tex.get_texture().get_id(), bind_point: gl::TEXTURE_2D, level: 0, layer: 0 }), Some(32))      // FIXME: wrong number
                },

                DepthAttachment::RenderBuffer(buffer) => {
                    // TODO: dimensions

                    (Some(fbo::Attachment::RenderBuffer(buffer.get_id())), Some(32))      // FIXME: wrong number
                },

                _ => unimplemented!()
            }

        } else {
            (None, None)
        };

        let (stencil, stencil_bits) = if let Some(stencil) = stencil {
            match stencil {
                StencilAttachment::Texture2d(tex) => {
                    if (tex.get_texture().get_width(), tex.get_texture().get_height().unwrap()) != dimensions {
                        panic!("The stencil attachment must have the same dimensions \
                                as the color attachment");
                    }

                    (Some(fbo::Attachment::Texture { id: tex.get_texture().get_id(), bind_point: gl::TEXTURE_2D, level: 0, layer: 0 }), Some(8))       // FIXME: wrong number
                },

                StencilAttachment::RenderBuffer(buffer) => {
                    // TODO: dimensions

                    (Some(fbo::Attachment::RenderBuffer(buffer.get_id())), Some(8))
                },

                _ => unimplemented!()
            }

        } else {
            (None, None)
        };

        SimpleFrameBuffer {
            context: facade.get_context().clone(),
            attachments: FramebufferAttachments {
                colors: vec![(0, color_attachment)],
                depth_stencil: if let (Some(depth), Some(stencil)) = (depth, stencil) {
                    fbo::FramebufferDepthStencilAttachments::DepthAndStencilAttachments(depth, stencil)
                } else if let Some(depth) = depth {
                    fbo::FramebufferDepthStencilAttachments::DepthAttachment(depth)
                } else if let Some(stencil) = stencil {
                    fbo::FramebufferDepthStencilAttachments::DepthAttachment(stencil)
                } else {
                    fbo::FramebufferDepthStencilAttachments::None
                },
            },
            marker: PhantomData,
            dimensions: dimensions,
            depth_buffer_bits: depth_bits,
            stencil_buffer_bits: stencil_bits,
        }
    }
}

impl<'a> Surface for SimpleFrameBuffer<'a> {
    fn clear(&mut self, color: Option<(f32, f32, f32, f32)>, depth: Option<f32>,
             stencil: Option<i32>)
    {
        ops::clear(&self.context, Some(&self.attachments), color, depth, stencil);
    }

    fn get_dimensions(&self) -> (u32, u32) {
        (self.dimensions.0 as u32, self.dimensions.1 as u32)
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.depth_buffer_bits
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.stencil_buffer_bits
    }

    fn draw<'b, 'v, V, I, U>(&mut self, vb: V, ib: &I, program: &::Program,
        uniforms: U, draw_parameters: &::DrawParameters) -> Result<(), DrawError>
        where I: ::index::ToIndicesSource, U: ::uniforms::Uniforms,
        V: ::vertex::MultiVerticesSource<'v>
    {
        use index::ToIndicesSource;

        if !self.has_depth_buffer() && (draw_parameters.depth_test.requires_depth_buffer() ||
                        draw_parameters.depth_write)
        {
            return Err(DrawError::NoDepthBuffer);
        }

        if let Some(viewport) = draw_parameters.viewport {
            if viewport.width > self.context.capabilities().max_viewport_dims.0
                    as u32
            {
                return Err(DrawError::ViewportTooLarge);
            }
            if viewport.height > self.context.capabilities().max_viewport_dims.1
                    as u32
            {
                return Err(DrawError::ViewportTooLarge);
            }
        }

        ops::draw(&self.context, Some(&self.attachments), vb,
                  ib.to_indices_source(), program, uniforms, draw_parameters, self.dimensions)
    }

    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        target.blit_from_simple_framebuffer(self, source_rect, target_rect, filter)
    }

    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, None, self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    fn blit_from_simple_framebuffer(&self, source: &SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    fn blit_from_multioutput_framebuffer(&self, source: &MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }
}

impl<'a> FboAttachments for SimpleFrameBuffer<'a> {
    fn get_attachments(&self) -> Option<&FramebufferAttachments> {
        Some(&self.attachments)
    }
}

/// This struct is useless for the moment.
pub struct MultiOutputFrameBuffer<'a> {
    context: Rc<Context>,
    marker: PhantomData<&'a ()>,
    dimensions: (u32, u32),
    color_attachments: Vec<(String, gl::types::GLuint)>,
    depth_attachment: Option<fbo::Attachment>,
    depth_buffer_bits: Option<u16>,
    stencil_attachment: Option<fbo::Attachment>,
    stencil_buffer_bits: Option<u16>,
}

impl<'a> MultiOutputFrameBuffer<'a> {
    /// Creates a new `MultiOutputFrameBuffer`.
    ///
    /// # Panic
    ///
    /// Panics if all attachments don't have the same dimensions.
    pub fn new<F>(facade: &F, color_attachments: &[(&str, &'a Texture2d)])
                  -> MultiOutputFrameBuffer<'a> where F: Facade
    {
        use render_buffer;

        MultiOutputFrameBuffer::new_impl(facade, color_attachments,
                                         None::<&render_buffer::DepthRenderBuffer>,
                                         None::<&render_buffer::StencilRenderBuffer>)
    }

    /// Creates a `MultiOutputFrameBuffer` with a depth buffer.
    ///
    /// # Panic
    ///
    /// Panics if all attachments don't have the same dimensions.
    pub fn with_depth_buffer<F, D>(facade: &F, color_attachments: &[(&str, &'a Texture2d)],
                                   depth: &'a D) -> MultiOutputFrameBuffer<'a>
                                   where D: ToDepthAttachment, F: Facade
    {
        use render_buffer;
        
        MultiOutputFrameBuffer::new_impl(facade, color_attachments, Some(depth),
                                         None::<&render_buffer::StencilRenderBuffer>)
    }

    fn new_impl<F, D, S>(facade: &F, color_attachments: &[(&str, &'a Texture2d)],
                         depth: Option<&'a D>, stencil: Option<&'a S>)
                         -> MultiOutputFrameBuffer<'a> where D: ToDepthAttachment, F: Facade
    {
        assert!(stencil.is_none());     // not implemented yet

        let mut attachments = Vec::new();
        let mut dimensions = None;

        for &(name, texture) in color_attachments.iter() {
            let tex_dims = (texture.get_width(), texture.get_height().unwrap());

            if let Some(ref dimensions) = dimensions {
                if dimensions != &tex_dims {
                    panic!("All textures of a MultiOutputFrameBuffer must have \
                            the same dimensions");
                }
            }

            dimensions = Some(tex_dims);
            attachments.push((name.to_string(), texture.get_id()));
        }

        let dimensions = match dimensions {
            None => panic!("Cannot pass an empty color_attachments when \
                            creating a MultiOutputFrameBuffer"),
            Some(d) => d
        };

        let (depth, depth_bits) = if let Some(depth) = depth {
            match depth.to_depth_attachment() {
                DepthAttachment::Texture2d(tex) => {
                    if (tex.get_texture().get_width(), tex.get_texture().get_height().unwrap()) != dimensions {
                        panic!("The depth attachment must have the same dimensions \
                                as the color attachment");
                    }

                    (Some(fbo::Attachment::Texture { id: tex.get_texture().get_id(), bind_point: gl::TEXTURE_2D, level: 0, layer: 0 }), Some(32))      // FIXME: wrong number
                },

                DepthAttachment::RenderBuffer(buffer) => {
                    // TODO: dimensions

                    (Some(fbo::Attachment::RenderBuffer(buffer.get_id())), Some(32))      // FIXME: wrong number
                },

                _ => unimplemented!()
            }

        } else {
            (None, None)
        };

        MultiOutputFrameBuffer {
            context: facade.get_context().clone(),
            marker: PhantomData,
            dimensions: dimensions,
            color_attachments: attachments,
            depth_attachment: depth,
            depth_buffer_bits: depth_bits,
            stencil_attachment: None,
            stencil_buffer_bits: None,
        }
    }

    fn build_attachments(&self, program: &Program) -> FramebufferAttachments {
        let mut colors = Vec::new();

        for &(ref name, texture) in self.color_attachments.iter() {
            let location = match program.get_frag_data_location(&name) {
                Some(l) => l,
                None => panic!("The fragment output `{}` was not found in the program", name)
            };

            colors.push((location, fbo::Attachment::Texture { id: texture, bind_point: gl::TEXTURE_2D, level: 0, layer: 0 }));
        }

        FramebufferAttachments {
            colors: colors,
            depth_stencil: if let Some(depth) = self.depth_attachment {
                fbo::FramebufferDepthStencilAttachments::DepthAttachment(depth)
            } else {
                fbo::FramebufferDepthStencilAttachments::None
            },
        }
    }

    fn build_attachments_any(&self) -> FramebufferAttachments {
        let mut colors = Vec::new();

        for (id, &(ref name, texture)) in self.color_attachments.iter().enumerate() {
            colors.push((id as u32, fbo::Attachment::Texture { id: texture, bind_point: gl::TEXTURE_2D, level: 0, layer: 0 }));
        }

        FramebufferAttachments {
            colors: colors,
            depth_stencil: if let Some(depth) = self.depth_attachment {
                fbo::FramebufferDepthStencilAttachments::DepthAttachment(depth)
            } else {
                fbo::FramebufferDepthStencilAttachments::None
            },
        }
    }
}

impl<'a> Surface for MultiOutputFrameBuffer<'a> {
    fn clear(&mut self, color: Option<(f32, f32, f32, f32)>, depth: Option<f32>,
             stencil: Option<i32>)
    {
        ops::clear(&self.context, Some(&self.build_attachments_any()),
                   color, depth, stencil);
    }

    fn get_dimensions(&self) -> (u32, u32) {
        (self.dimensions.0 as u32, self.dimensions.1 as u32)
    }

    fn get_depth_buffer_bits(&self) -> Option<u16> {
        self.depth_buffer_bits
    }

    fn get_stencil_buffer_bits(&self) -> Option<u16> {
        self.stencil_buffer_bits
    }

    fn draw<'v, V, I, U>(&mut self, vb: V, ib: &I, program: &::Program,
        uniforms: U, draw_parameters: &::DrawParameters) -> Result<(), DrawError>
        where I: ::index::ToIndicesSource,
        U: ::uniforms::Uniforms, V: ::vertex::MultiVerticesSource<'v>
    {
        use index::ToIndicesSource;

        if !self.has_depth_buffer() && (draw_parameters.depth_test.requires_depth_buffer() ||
                draw_parameters.depth_write)
        {
            return Err(DrawError::NoDepthBuffer);
        }

        if let Some(viewport) = draw_parameters.viewport {
            if viewport.width > self.context.capabilities().max_viewport_dims.0
                    as u32
            {
                return Err(DrawError::ViewportTooLarge);
            }
            if viewport.height > self.context.capabilities().max_viewport_dims.1
                    as u32
            {
                return Err(DrawError::ViewportTooLarge);
            }
        }

        ops::draw(&self.context, Some(&self.build_attachments(program)), vb,
                  ib.to_indices_source(), program, uniforms, draw_parameters, self.dimensions)
    }

    fn blit_color<S>(&self, source_rect: &Rect, target: &S, target_rect: &BlitTarget,
                     filter: uniforms::MagnifySamplerFilter) where S: Surface
    {
        target.blit_from_multioutput_framebuffer(self, source_rect, target_rect, filter)
    }

    fn blit_from_frame(&self, source_rect: &Rect, target_rect: &BlitTarget,
                       filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, None, self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    fn blit_from_simple_framebuffer(&self, source: &SimpleFrameBuffer,
                                    source_rect: &Rect, target_rect: &BlitTarget,
                                    filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }

    fn blit_from_multioutput_framebuffer(&self, source: &MultiOutputFrameBuffer,
                                         source_rect: &Rect, target_rect: &BlitTarget,
                                         filter: uniforms::MagnifySamplerFilter)
    {
        ops::blit(&self.context, source.get_attachments(), self.get_attachments(),
                  gl::COLOR_BUFFER_BIT, source_rect, target_rect, filter.to_glenum())
    }
}

impl<'a> FboAttachments for MultiOutputFrameBuffer<'a> {
    fn get_attachments(&self) -> Option<&FramebufferAttachments> {
        unimplemented!();
    }
}

/// Describes an attachment for a color buffer.
#[derive(Copy, Clone)]
pub enum ColorAttachment<'a> {
    /// A texture.
    Texture1d(Texture1dMipmap<'a>),
    /// A texture.
    SrgbTexture1d(SrgbTexture1dMipmap<'a>),
    /// A texture.
    Texture2d(Texture2dMipmap<'a>),
    /// A texture.
    SrgbTexture2d(SrgbTexture2dMipmap<'a>),
    /// A texture.
    Texture2dMultisample(Texture2dMultisampleMipmap<'a>),
    /// A texture.
    SrgbTexture2dMultisample(SrgbTexture2dMultisampleMipmap<'a>),
    /// A texture.
    Texture3d(Texture3dMipmap<'a>, u32),
    /// A texture.
    SrgbTexture3d(SrgbTexture3dMipmap<'a>, u32),
    /// A texture.
    Texture1dArray(Texture1dArrayMipmap<'a>),
    /// A texture.
    SrgbTexture1dArray(SrgbTexture1dArrayMipmap<'a>),
    /// A texture.
    Texture2dArray(Texture2dArrayMipmap<'a>),
    /// A texture.
    SrgbTexture2dArray(SrgbTexture2dArrayMipmap<'a>),
    /// A texture.
    Texture2dMultisampleArray(Texture2dMultisampleArrayMipmap<'a>),
    /// A texture.
    SrgbTexture2dMultisampleArray(SrgbTexture2dMultisampleArrayMipmap<'a>),
    /// A render buffer.
    RenderBuffer(&'a ::render_buffer::RenderBuffer),
}

/// Trait for objects that can be used as color attachments.
pub trait ToColorAttachment {
    /// Builds the `ColorAttachment`.
    fn to_color_attachment(&self) -> ColorAttachment;
}

/// Describes an attachment for a depth buffer.
#[derive(Copy, Clone)]
pub enum DepthAttachment<'a> {
    /// A texture.
    Texture1d(DepthTexture1dMipmap<'a>),
    /// A texture.
    Texture2d(DepthTexture2dMipmap<'a>),
    /// A texture.
    Texture2dMultisample(DepthTexture2dMultisampleMipmap<'a>),
    /// A texture.
    Texture3d(DepthTexture3dMipmap<'a>, u32),
    /// A texture.
    Texture1dArray(DepthTexture1dArrayMipmap<'a>),
    /// A texture.
    Texture2dArray(DepthTexture2dArrayMipmap<'a>),
    /// A texture.
    Texture2dMultisampleArray(DepthTexture2dMultisampleArrayMipmap<'a>),
    /// A render buffer.
    RenderBuffer(&'a ::render_buffer::DepthRenderBuffer),
}

/// Trait for objects that can be used as depth attachments.
pub trait ToDepthAttachment {
    /// Builds the `DepthAttachment`.
    fn to_depth_attachment(&self) -> DepthAttachment;
}

/// Describes an attachment for a stencil buffer.
#[derive(Copy, Clone)]
pub enum StencilAttachment<'a> {
    /// A texture.
    Texture1d(StencilTexture1dMipmap<'a>),
    /// A texture.
    Texture2d(StencilTexture2dMipmap<'a>),
    /// A texture.
    Texture2dMultisample(StencilTexture2dMultisampleMipmap<'a>),
    /// A texture.
    Texture3d(StencilTexture3dMipmap<'a>, u32),
    /// A texture.
    Texture1dArray(StencilTexture1dArrayMipmap<'a>),
    /// A texture.
    Texture2dArray(StencilTexture2dArrayMipmap<'a>),
    /// A texture.
    Texture2dMultisampleArray(StencilTexture2dMultisampleArrayMipmap<'a>),
    /// A render buffer.
    RenderBuffer(&'a ::render_buffer::StencilRenderBuffer),
}

/// Trait for objects that can be used as stencil attachments.
pub trait ToStencilAttachment {
    /// Builds the `StencilAttachment`.
    fn to_stencil_attachment(&self) -> StencilAttachment;
}

/// Describes an attachment for a depth and stencil buffer.
#[derive(Copy, Clone)]
pub enum DepthStencilAttachment<'a> {
    /// A texture.
    Texture1d(DepthStencilTexture1dMipmap<'a>),
    /// A texture.
    Texture2d(DepthStencilTexture2dMipmap<'a>),
    /// A texture.
    Texture2dMultisample(DepthStencilTexture2dMultisampleMipmap<'a>),
    /// A texture.
    Texture3d(DepthStencilTexture3dMipmap<'a>, u32),
    /// A texture.
    Texture1dArray(DepthStencilTexture1dArrayMipmap<'a>),
    /// A texture.
    Texture2dArray(DepthStencilTexture2dArrayMipmap<'a>),
    /// A texture.
    Texture2dMultisampleArray(DepthStencilTexture2dMultisampleArrayMipmap<'a>),
    /// A render buffer.
    RenderBuffer(&'a ::render_buffer::DepthStencilRenderBuffer),
}

/// Trait for objects that can be used as depth and stencil attachments.
pub trait ToDepthStencilAttachment {
    /// Builds the `DepthStencilAttachment`.
    fn to_depth_stencil_attachment(&self) -> DepthStencilAttachment;
}
