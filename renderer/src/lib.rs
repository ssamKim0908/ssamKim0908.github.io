use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlShader};

const VERT_SRC: &str = r#"#version 300 es
in vec3 a_position;
in vec3 a_normal;
uniform mat4 u_mvp;
uniform mat4 u_model;
out vec3 v_normal;
void main() {
    gl_Position = u_mvp * vec4(a_position, 1.0);
    v_normal = mat3(u_model) * a_normal;
}
"#;

const FRAG_SRC: &str = r#"#version 300 es
precision mediump float;
in vec3 v_normal;
out vec4 fragColor;
void main() {
    vec3 n = normalize(v_normal);
    vec3 l = normalize(vec3(0.5, 0.8, 0.6));
    float diff = max(dot(n, l), 0.0);
    vec3 base = vec3(0.35, 0.62, 0.95);
    vec3 color = base * (0.25 + 0.75 * diff);
    fragColor = vec4(color, 1.0);
}
"#;

fn compile(gl: &GL, kind: u32, src: &str) -> Result<WebGlShader, String> {
    let sh = gl.create_shader(kind).ok_or("create_shader failed")?;
    gl.shader_source(&sh, src);
    gl.compile_shader(&sh);
    let ok = gl
        .get_shader_parameter(&sh, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false);
    if ok {
        Ok(sh)
    } else {
        Err(gl.get_shader_info_log(&sh).unwrap_or_default())
    }
}

fn link(gl: &GL, vs: &WebGlShader, fs: &WebGlShader) -> Result<WebGlProgram, String> {
    let p = gl.create_program().ok_or("create_program failed")?;
    gl.attach_shader(&p, vs);
    gl.attach_shader(&p, fs);
    gl.link_program(&p);
    let ok = gl
        .get_program_parameter(&p, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false);
    if ok {
        Ok(p)
    } else {
        Err(gl.get_program_info_log(&p).unwrap_or_default())
    }
}

fn uv_sphere(segments: u32, rings: u32) -> (Vec<f32>, Vec<u16>) {
    let mut v = Vec::with_capacity(((segments + 1) * (rings + 1) * 6) as usize);
    let mut i = Vec::with_capacity((segments * rings * 6) as usize);
    for ring in 0..=rings {
        let theta = ring as f32 * std::f32::consts::PI / rings as f32;
        let (st, ct) = (theta.sin(), theta.cos());
        for seg in 0..=segments {
            let phi = seg as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let (sp, cp) = (phi.sin(), phi.cos());
            let x = cp * st;
            let y = ct;
            let z = sp * st;
            v.extend_from_slice(&[x, y, z, x, y, z]);
        }
    }
    for ring in 0..rings {
        for seg in 0..segments {
            let a = ring * (segments + 1) + seg;
            let b = a + segments + 1;
            i.extend_from_slice(&[
                a as u16,
                b as u16,
                (a + 1) as u16,
                b as u16,
                (b + 1) as u16,
                (a + 1) as u16,
            ]);
        }
    }
    (v, i)
}

fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fovy * 0.5).tan();
    let nf = 1.0 / (near - far);
    [
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, (far + near) * nf, -1.0,
        0.0, 0.0, 2.0 * far * near * nf, 0.0,
    ]
}

fn rotate_y(a: f32) -> [f32; 16] {
    let (s, c) = (a.sin(), a.cos());
    [
        c, 0.0, -s, 0.0,
        0.0, 1.0, 0.0, 0.0,
        s, 0.0, c, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn translate(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        x, y, z, 1.0,
    ]
}

fn mat_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut r = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut s = 0.0;
            for k in 0..4 {
                s += a[k * 4 + row] * b[col * 4 + k];
            }
            r[col * 4 + row] = s;
        }
    }
    r
}

#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str("canvas not found"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl = canvas
        .get_context("webgl2")?
        .ok_or_else(|| JsValue::from_str("webgl2 unavailable"))?
        .dyn_into::<GL>()?;

    let vs = compile(&gl, GL::VERTEX_SHADER, VERT_SRC).map_err(|e| JsValue::from_str(&e))?;
    let fs = compile(&gl, GL::FRAGMENT_SHADER, FRAG_SRC).map_err(|e| JsValue::from_str(&e))?;
    let program = link(&gl, &vs, &fs).map_err(|e| JsValue::from_str(&e))?;
    gl.use_program(Some(&program));

    let (verts, idx) = uv_sphere(32, 16);

    let vao = gl
        .create_vertex_array()
        .ok_or_else(|| JsValue::from_str("create_vertex_array"))?;
    gl.bind_vertex_array(Some(&vao));

    let vbo = gl
        .create_buffer()
        .ok_or_else(|| JsValue::from_str("create_buffer vbo"))?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
    unsafe {
        let view = js_sys::Float32Array::view(&verts);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
    }

    let pos_loc = gl.get_attrib_location(&program, "a_position") as u32;
    gl.vertex_attrib_pointer_with_i32(pos_loc, 3, GL::FLOAT, false, 6 * 4, 0);
    gl.enable_vertex_attrib_array(pos_loc);
    let nrm_loc = gl.get_attrib_location(&program, "a_normal") as u32;
    gl.vertex_attrib_pointer_with_i32(nrm_loc, 3, GL::FLOAT, false, 6 * 4, 3 * 4);
    gl.enable_vertex_attrib_array(nrm_loc);

    let ebo = gl
        .create_buffer()
        .ok_or_else(|| JsValue::from_str("create_buffer ebo"))?;
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ebo));
    unsafe {
        let view = js_sys::Uint16Array::view(&idx);
        gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &view, GL::STATIC_DRAW);
    }

    let u_mvp = gl.get_uniform_location(&program, "u_mvp");
    let u_model = gl.get_uniform_location(&program, "u_model");

    gl.enable(GL::DEPTH_TEST);
    gl.clear_color(0.07, 0.09, 0.12, 1.0);

    let idx_count = idx.len() as i32;
    let perf = window
        .performance()
        .ok_or_else(|| JsValue::from_str("no performance"))?;
    let start_time = perf.now();

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let win = window.clone();

    *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        let w = canvas.client_width().max(1);
        let h = canvas.client_height().max(1);
        if canvas.width() != w as u32 {
            canvas.set_width(w as u32);
        }
        if canvas.height() != h as u32 {
            canvas.set_height(h as u32);
        }
        gl.viewport(0, 0, w, h);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let t = ((perf.now() - start_time) / 1000.0) as f32;
        let model = rotate_y(t * 0.8);
        let view = translate(0.0, 0.0, -3.0);
        let mv = mat_mul(&view, &model);
        let proj = perspective(45.0_f32.to_radians(), w as f32 / h as f32, 0.1, 100.0);
        let mvp = mat_mul(&proj, &mv);

        gl.uniform_matrix4fv_with_f32_array(u_mvp.as_ref(), false, &mvp);
        gl.uniform_matrix4fv_with_f32_array(u_model.as_ref(), false, &model);

        gl.draw_elements_with_i32(GL::TRIANGLES, idx_count, GL::UNSIGNED_SHORT, 0);

        win.request_animation_frame(
            f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
        )
        .unwrap();
    }));

    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;

    Ok(())
}
