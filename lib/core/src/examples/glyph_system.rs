use crate::prelude::*;

use basegl_system_web::{forward_panic_hook_to_console, set_stdout};
use crate::display::world::*;
use crate::display::object::DisplayObjectOps;
use crate::display::shape::glyphs::GlyphSystem;
use nalgebra::Vector4;

use wasm_bindgen::prelude::*;
use basegl_core_msdf_sys::run_once_initialized;

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_glyph_system() {
    forward_panic_hook_to_console();
    set_stdout();
    WorldData::new("canvas");
    run_once_initialized(init);
}

fn init() {
    let font_id          = get_world().rc.borrow_mut().fonts.load_embedded_font("DejaVuSansMono").unwrap();
    let mut glyph_system = GlyphSystem::new(font_id);
    let letter_a         = glyph_system.new_glyph('A', Vector4::new(0.0, 1.0, 0.0, 1.0), &mut get_world().rc.borrow_mut().fonts);
    let letter_b         = glyph_system.new_glyph('b', Vector4::new(1.0, 0.0, 0.0, 1.0), &mut get_world().rc.borrow_mut().fonts);
    let letter_c         = glyph_system.new_glyph('C', Vector4::new(0.0, 0.0, 2.0, 1.0), &mut get_world().rc.borrow_mut().fonts);
    letter_a.size().set(Vector2::new(50.0,30.0));
    letter_b.size().set(Vector2::new(50.0,30.0));
    letter_c.size().set(Vector2::new(50.0,30.0));
    letter_a.set_position(Vector3::new(100.0, 100.0, 1.0));
    letter_b.set_position(Vector3::new(150.0, 100.0, 1.0));
    letter_c.set_position(Vector3::new(200.0, 100.0, 1.0));

    get_world().add_child(glyph_system.sprite_system());
    get_world().on_frame(move |_| {
        let &_ = &letter_a;
        let &_ = &letter_b;
        let &_ = &letter_c;
        glyph_system.sprite_system().update()
    }).forget();
}