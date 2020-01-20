// TODO [ao] name of this module should be rather refactored

use crate::display::symbol::material::Material;
use crate::display::shape::text::font::{FontId, FontRenderInfo, Fonts};
use nalgebra::Vector2;
use nalgebra::Vector4;
use crate::display::world::*;
use crate::system::gpu::types::Context;
use crate::display::symbol::SpriteSystem;
use crate::system::gpu::data::texture::{Rgb, Texture, GpuOnly, GpuOnlyData, MemoryView, OwnedData};
use crate::display::world;
use crate::display::symbol::shader::builder::CodeTemplete;
use crate::system::gpu::data::uniform::AnyTextureUniform;
use crate::display::shape::text::msdf::MsdfTexture;


pub struct GlyphSystem {
    sprite_system      : SpriteSystem,
    pub font_id            : FontId,
    color                  : Buffer<Vector4<f32>>,
    glyph_msdf_index       : Buffer<f32>,
    msdf_uniform           : Uniform<Texture<MemoryView,Rgb,u8>>,
}

impl GlyphSystem {
    /// Constructor.
    pub fn new(font_id:FontId) -> Self {
        let mut world         = get_world();
        let workspace         = world.workspace();
        let context           = workspace.context();
        let mut sprite_system = SpriteSystem::new();

        sprite_system.set_material(Self::material(&context));
        workspace.variables().add("msdf_range", FontRenderInfo::MSDF_PARAMS.range as f32);
        workspace.variables().add("msdf_size", Vector2::new(MsdfTexture::WIDTH as f32,MsdfTexture::ONE_GLYPH_HEIGHT as f32));
        let symbol       = sprite_system.symbol();
        let texture      = Texture::<MemoryView,Rgb,u8>::new(&context,&[],0,0);
        let msdf_uniform = symbol.variables().add_or_panic("msdf_texture",texture);
        let mesh         = symbol.surface();

        Self {sprite_system,font_id,msdf_uniform,
            color              : mesh.instance_scope().add_buffer("color"),
            glyph_msdf_index   : mesh.instance_scope().add_buffer("glyph_msdf_index"),
        }
    }

    pub fn new_glyph(&mut self, ch:char, color:Vector4<f32>, fonts:&mut Fonts) -> Sprite {
        let sprite                = self.sprite_system.new_instance();
        let instance_id           = sprite.instance_id();
        let color_attr            = self.color.at(instance_id);
        let glyph_msdf_index_attr = self.glyph_msdf_index.at(instance_id);

        color_attr.set(color);
        let font       = fonts.get_render_info(self.font_id);
        let glyph_info = font.get_glyph_info(ch);
        let msdf_index = glyph_info.msdf_texture_glyph_id;
        glyph_msdf_index_attr.set(msdf_index as f32);

        self.msdf_uniform.modify(|texture| {
            if texture.height() != font.msdf_texture.rows() as i32 {
                let data   = font.msdf_texture.data.as_slice();
                let width  = MsdfTexture::WIDTH       as i32;
                let height = font.msdf_texture.rows() as i32;
                texture.reload(data,width,height);
            }
        });
        sprite
    }

    pub fn sprite_system(&self) -> &SpriteSystem {
        &self.sprite_system
    }

    /// Defines a default material of this system.
    fn material(context:&Context) -> Material {
        let mut material = Material::new();
        material.add_input("pixel_ratio"  , 1.0);
        material.add_input("zoom"         , 1.0);
        material.add_input_def::<GpuOnlyData>("msdf_texture");
        material.add_input_def::<Vector2<f32>>("msdf_size");
        material.add_input_def::<f32>("glyph_msdf_index");
        material.add_input("msdf_range"   , FontRenderInfo::MSDF_PARAMS.range as f32);
        material.add_input("color"        , Vector4::new(1.0,1.0,1.0,1.0));

        // TODO[AO] rename CodeTemplete->CodeTemplate once PR will be ready.
        let code = CodeTemplete::new(BEFORE_MAIN.to_string(),MAIN.to_string(),"".to_string());
        material.set_code(code);
        material
    }
}

const BEFORE_MAIN : &str = include_str!("glyphs/glyph.glsl");
const MAIN        : &str = "output_color = color_from_msdf();";
