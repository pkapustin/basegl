function arr_to_css_matrix3d(a) {
    return "matrix3d(" + a.join(',') + ")"
}

// Sets object's CSS 3D transform.
export function set_object_transform(dom, matrix_array) {
    let css = arr_to_css_matrix3d(matrix_array);
    dom.style.transform = "translate(-50%, -50%)" + css;
}

// Setup perspective CSS 3D projection on DOM.
export function setup_perspective(dom, perspective) {
    dom.style.perspective = perspective + "px";
}

// Setup Camera orthographic projection on DOM.
export function setup_camera_orthographic(dom, matrix_array) {
    dom.style.transform = arr_to_css_matrix3d(matrix_array);
}

// Setup Camera perspective projection on DOM.
export function setup_camera_perspective
(dom, y_scale, half_width, half_height, matrix_array) {
    let translateZ  = "translateZ(" + y_scale + "px)";
    let matrix3d    = arr_to_css_matrix3d(matrix_array);
    let translate2d = "translate(" + half_width + "px," + half_height + "px)";
    let transform   = translateZ + matrix3d + translate2d;
    dom.style.transform = transform;
}
