use crate::prelude::*;

use crate::text::content::CharPosition;
use crate::text::content::TextComponentContent;
use crate::text::content::line::LineRef;
use crate::text::buffer::glyph_square::point_to_iterable;
use crate::text::font::Fonts;

use basegl_backend_webgl::Context;
use basegl_backend_webgl::set_buffer_data;
use nalgebra::Point2;
use nalgebra::Translation2;
use std::collections::HashSet;
use std::cmp::Ordering;
use std::iter::once;
use std::ops::Range;
use std::ops::RangeBounds;
use std::ops::RangeInclusive;
use web_sys::WebGlBuffer;


// ==============
// === Cursor ===
// ==============

/// Cursor in TextComponent with its selection
#[derive(Debug)]
pub struct Cursor {
    pub position    : CharPosition,
    pub selected_to : CharPosition,
}

impl Cursor {
    /// Create a new cursor at given position and without any selection.
    pub fn new(position:CharPosition) -> Self {
        Cursor {position,
            selected_to : position
        }
    }

    /// Get range of selected text by this cursor.
    pub fn selection_range(&self) -> Range<CharPosition> {
        match self.position.cmp(&self.selected_to) {
            Ordering::Equal   => self.position..self.position,
            Ordering::Greater => self.selected_to..self.position,
            Ordering::Less    => self.position..self.selected_to
        }
    }

    /// Check if char at given position is selected.
    pub fn is_char_selected(&self, position:CharPosition) -> bool {
        self.selection_range().contains(&position)
    }

    /// Get `LineRef` object of this cursor's line.
    pub fn current_line<'a>(&self, content:&'a mut TextComponentContent) -> LineRef<'a> {
        content.line(self.position.column)
    }

    /// Get the position where the cursor should be rendered. The returned point is on the
    /// _baseline_ of cursor's line, on the right side of character from the left side of the cursor
    /// (where usually the cursor is displayed by text editors).
    ///
    /// _Baseline_ is a font specific term, for details see [freetype documentation]
    //  (https://www.freetype.org/freetype2/docs/glyphs/glyphs-3.html#section-1).
    pub fn render_position(&self, content:&mut TextComponentContent, fonts:&mut Fonts)
    -> Point2<f64>{
        let font     = fonts.get_render_info(content.font);
        let mut line = self.current_line(content);
        if self.position.column > 0 {
            let char_index = self.position.column - 1;
            let x          = line.get_char_x_range(char_index,font).end;
            let y          = line.start_point().y;
            Point2::new(x.into(),y)
        } else {
            line.start_point()
        }
    }
}


// ===============
// === Cursors ===
// ===============

/// The number of vertices of single cursor.
const VERTICES_PER_CURSOR : usize = 2;
const LINE_TOP            : f64   = 0.8;
const LINE_BOTTOM         : f64   = -0.2;


/// Structure handling many cursors.
///
/// Usually there is only one cursor, but we have possibility of having many cursors in one text
/// component enabling editing in multiple lines/places at once. This structure also owns
/// a WebGL buffer with vertex positions of all cursors.
#[derive(Debug)]
pub struct Cursors {
    pub cursors          : Vec<Cursor>,
    pub cursors_dirty    : bool,
    pub selection_dirty  : bool,
    pub cursors_buffer   : WebGlBuffer,
    pub selection_buffer : WebGlBuffer,
}

impl Cursors {

    /// Create empty `Cursors` structure.
    pub fn new(gl_context:&Context) -> Self {
        Cursors {
            cursors          : Vec::new(),
            cursors_dirty    : bool,
            selection_dirty  : bool,
            cursors_buffer   : gl_context.create_buffer().unwrap(),
            selection_buffer : gl_context.create_buffer().unwrap(),
        }
    }

    /// Removes all current cursors and replace them with single cursor without any selection.
    pub fn set_cursor(&mut self, position:CharPosition) {
        self.cursors       = vec![Cursor::new(position)];
        self.dirty_cursors = once(0).collect();
    }

    /// Add new cursor without selection.
    pub fn add_cursor(&mut self, position:CharPosition) {
        let new_index = self.cursors.len();
        self.cursors.push(Cursor::new(position));
        self.dirty_cursors.insert(new_index);
    }

    /// Checks if cursors should be currently visible as a part of their blinking.
    pub fn cursors_should_be_visible() -> bool {
        const BLINKING_PERIOD_MS : f64 = 1000.0;
        let timestamp                  = js_sys::Date::now();
        let blinks                     = timestamp / BLINKING_PERIOD_MS;
        blinks.fract() >= 0.5
    }

    /// Update the cursors' buffer data.
    pub fn update_buffer_data
    (&mut self, gl_context:&Context, content:&mut TextComponentContent, fonts:&mut Fonts) {
        let cursors          = self.cursors.iter();
        let cursors_vertices = cursors.map(|cursor| Self::cursor_vertices(cursor,content,fonts));
        let buffer_data      = cursors_vertices.flatten().collect_vec();
        set_buffer_data(gl_context,&self.buffer,buffer_data.as_slice());
    }

    fn cursor_vertices(cursor:&Cursor, content:&mut TextComponentContent, fonts:&mut Fonts)
    -> SmallVec<[f32;4]> {
        let position = cursor.render_position(content,fonts);
        let x        = position.x as f32;
        let y_min    = (position.y + LINE_BOTTOM) as f32;
        let y_max    = (position.y + LINE_TOP)    as f32;
        SmallVec::from_buf([x, y_min, x, y_max])
    }

    fn selection_vertices(cursor:&Cursor, content:&mut TextComponentContent, fonts:&mut Fonts)
    -> SmallVec<[f32;36]> {
        let selection         = cursor.selection_range();
        let font              = fonts.get_render_info(content.font);
        let left              = line.line.get_char_x_position(selection.start.column,font);
        let right             = line.line.get_char_x_range(selection.end.column,font).end;
        let min               = -1e30;
        let max               = 1e30;
        let first_line_top    = content.line(selection.start.line).start_point().y + LINE_TOP;
        let first_line_bottom = content.line(selection.start.line).start_point().y + LINE_BOTTOM;
        let last_line_top     = content.line(selection.end.line  ).start_point().y + LINE_TOP;
        let last_line_bottom  = content.line(selection.end.line  ).start_point().y + LINE_BOTTOM;
        if selection.start.line == selection.end.line {
            Self::vertices_of_square(left,right,first_line_top,first_line_bottom).into()
        } else {
            let first_sq  = Self::vertices_of_square(left,max ,first_line_top   ,first_line_bottom);
            let middle_sq = Self::vertices_of_square(min ,max ,first_line_bottom,last_line_top    );
            let last_sq   = Self::vertices_of_square(min ,right,last_line_top   ,last_line_bottom );
            [first_sq,middle_sq,last_sq].iter().flatten().collect()
        }
    }

    fn vertices_of_square<F>(left:F, right:F, top:F, bottom:F)
    -> SmallVec<[f32;12]>
    where F : Into<f32> {
        SmallVec::from_buf([left,bottom,left,top,right,bottom,left,top,right,bottom,right,top])
    }

    /// Number of vertices in cursors' buffer.
    pub fn vertices_count(&self) -> usize {
        self.cursors.len() * VERTICES_PER_CURSOR
    }
}
