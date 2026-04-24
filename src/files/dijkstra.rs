#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

fn __fractal_fmt_float(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

use std::sync::{Mutex, Once};
use std::fs::{OpenOptions, File as __DbgFile};
use std::io::{BufWriter as __DbgBufWriter, Write as __DbgWrite};

static __FRACTAL_DBG_INIT: Once = Once::new();
#[allow(clippy::type_complexity)]
static __FRACTAL_DBG_FILE: Mutex<Option<__DbgBufWriter<__DbgFile>>> = Mutex::new(None);
static __FRACTAL_DBG_PREV: std::sync::OnceLock<Mutex<std::collections::HashMap<String, String>>> = std::sync::OnceLock::new();
static __FRACTAL_DBG_STEP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static __FRACTAL_DBG_LOCK: std::sync::OnceLock<String> = std::sync::OnceLock::new();

thread_local! {
    static __FRACTAL_CALL_STACK: std::cell::RefCell<Vec<(String, String)>> =
        std::cell::RefCell::new(vec![("<main>".to_string(), "[]".to_string())]);
}

fn __fractal_debug_init() {
    __FRACTAL_DBG_INIT.call_once(|| {
        let __f = OpenOptions::new().create(true).write(true).truncate(true).open("/home/harikrishnanr/Code/Fractal/src/files/dijkstra.debug.jsonl").expect("cannot open fractal debug file");
        *__FRACTAL_DBG_FILE.lock().unwrap() = Some(__DbgBufWriter::new(__f));
    });
}

fn __fractal_debug_wait() {
    if let Some(path) = __FRACTAL_DBG_LOCK.get() {
        let _ = std::fs::write(path, "");
        while std::fs::metadata(path).is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

fn __fractal_debug_json_escape(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        match c {
            '"'  => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\t' => o.push_str("\\t"),
            '\r' => o.push_str("\\r"),
            c    => o.push(c),
        }
    }
    o
}

fn __fractal_debug_var(name: &str, type_label: &str, value: &str) -> String {
    let changed = {
        let mutex = __FRACTAL_DBG_PREV.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        let mut prev_map = mutex.lock().unwrap();
        let prev = prev_map.get(name).cloned().unwrap_or_default();
        let did_change = value != prev.as_str();
        prev_map.insert(name.to_string(), value.to_string());
        did_change
    };
    let mut s = String::from("{");
    s.push_str("\"name\":\""); s.push_str(&__fractal_debug_json_escape(name)); s.push_str("\",");
    s.push_str("\"type\":\""); s.push_str(&__fractal_debug_json_escape(type_label)); s.push_str("\",");
    s.push_str("\"value\":\""); s.push_str(&__fractal_debug_json_escape(value)); s.push_str("\",");
    s.push_str("\"changed\":"); s.push_str(if changed { "true" } else { "false" }); s.push('}');
    s
}

macro_rules! __fractal_debug_snapshot {
    ($label:expr, $func:expr, $line:expr, $file:expr, [$($var_str:expr),* $(,)?], $finished:expr, $error:expr) => {{
        let __dbg_step = __FRACTAL_DBG_STEP.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut __dbg_g = __FRACTAL_DBG_FILE.lock().unwrap();
        if let Some(ref mut __dbg_w) = *__dbg_g {
            let __dbg_vars: Vec<String> = vec![$($var_str),*];
            let __dbg_vars_json = format!("[{}]", __dbg_vars.join(","));
            __FRACTAL_CALL_STACK.with(|__stk| {
                if let Some(__top) = __stk.borrow_mut().last_mut() {
                    __top.1 = __dbg_vars_json.clone();
                }
            });
            let (__dbg_stack_json, __dbg_scopes_json) = __FRACTAL_CALL_STACK.with(|__stk| {
                let __frames = __stk.borrow();
                let __stack_parts: Vec<String> = __frames.iter()
                    .map(|(__name, _)| format!("\"{}\"" , __fractal_debug_json_escape(__name)))
                    .collect();
                let __scope_parts: Vec<String> = __frames.iter().rev()
                    .map(|(__name, __vars)| {
                        let mut __sc = String::from("{\"label\":\"");
                        __sc.push_str(&__fractal_debug_json_escape(__name));
                        __sc.push_str("\",\"vars\":");
                        __sc.push_str(__vars);
                        __sc.push('}');
                        __sc
                    })
                    .collect();
                (format!("[{}]", __stack_parts.join(",")),
                 format!("[{}]", __scope_parts.join(",")))
            });
            let __dbg_err: String = match ($error as Option<&str>) {
                None      => "null".into(),
                Some(__e) => { let mut __es = String::from("\""); __es.push_str(&__fractal_debug_json_escape(__e)); __es.push('"'); __es },
            };
            let __dbg_line = {
                let mut __ln = String::from("{\"step\":");
                __ln.push_str(&__dbg_step.to_string());
                __ln.push_str(",\"label\":\"");
                __ln.push_str(&__fractal_debug_json_escape($label));
                __ln.push_str("\",\"line\":");
                __ln.push_str(&($line as usize).to_string());
                __ln.push_str(",\"file\":\"");
                __ln.push_str(&__fractal_debug_json_escape($file));
                __ln.push_str("\"");
                __ln.push_str(",\"stack\":");
                __ln.push_str(&__dbg_stack_json);
                __ln.push_str(",\"scopes\":");
                __ln.push_str(&__dbg_scopes_json);
                __ln.push_str(",\"output\":\"\",\"finished\":");
                __ln.push_str(if $finished { "true" } else { "false" });
                __ln.push_str(",\"error\":");
                __ln.push_str(&__dbg_err);
                __ln.push('}');
                __ln
            };
            let _ = writeln!(__dbg_w, "{}", __dbg_line);
            let _ = __dbg_w.flush();
        }
    }};
}

#[derive(Debug, Clone, Default)]
pub struct FractalEdge {
    pub to: Option<i64>,
    pub weight: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct FractalGraph {
    pub n: Option<i64>,
    pub adj: Option<Vec<Vec<Option<Box<FractalEdge>>>>>,
}

pub fn fractal_make_graph(mut fractal_n: i64) -> Option<Box<FractalGraph>> {
    __fractal_debug_init();
    __FRACTAL_CALL_STACK.with(|__stk| { if let Some(__top) = __stk.borrow_mut().last_mut() { __top.1 = { let __cv: Vec<String> = vec![__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) })]; format!("[{}]", __cv.join(",")) }; } });
    __FRACTAL_CALL_STACK.with(|__s| __s.borrow_mut().push((format!("make_graph({:?})", fractal_n), { let __iv: Vec<String> = vec![__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) })]; format!("[{}]", __iv.join(",")) })));
    let mut fractal_g: Option<Box<FractalGraph>> = Some(Box::new(FractalGraph { n: Some(0), adj: Some(Vec::new()) }));
    __fractal_debug_snapshot!("StructDecl g : Graph", "make_graph", 11, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_g.as_mut().unwrap().n = Some(fractal_n);
    __fractal_debug_snapshot!("Assign Eq", "make_graph", 12, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_g.as_mut().unwrap().adj = Some(Vec::new());
    __fractal_debug_snapshot!("Assign Eq", "make_graph", 13, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    {
        let mut fractal_i: i64 = 0_i64;
        __fractal_debug_snapshot!("For i", "make_graph", 14, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        while fractal_i < fractal_n {
            let mut fractal_row: Vec<Option<Box<FractalEdge>>> = Vec::new();
            __fractal_debug_snapshot!("Decl row : :list =", "make_graph", 15, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_row", ":list", &{ let __v = &fractal_row; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_g.as_mut().unwrap().adj.as_mut().unwrap().push(fractal_row.clone());
            __fractal_debug_snapshot!("Call append", "make_graph", 16, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_row", ":list", &{ let __v = &fractal_row; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_i += 1_i64;
        }
    }
    __fractal_debug_snapshot!("For i", "make_graph", 14, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    __fractal_debug_snapshot!("Return", "make_graph", 18, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    return fractal_g.clone();
    __FRACTAL_CALL_STACK.with(|__s| { __s.borrow_mut().pop(); });
}

pub fn fractal_add_edge(mut fractal_g: &mut Option<Box<FractalGraph>>, mut fractal_u: i64, mut fractal_v: i64, mut fractal_w: i64) {
    __fractal_debug_init();
    __FRACTAL_CALL_STACK.with(|__stk| { if let Some(__top) = __stk.borrow_mut().last_mut() { __top.1 = { let __cv: Vec<String> = vec![__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_v", ":int", &{ let __v = &fractal_v; format!("{:?}", __v) }), __fractal_debug_var("fractal_w", ":int", &{ let __v = &fractal_w; format!("{:?}", __v) })]; format!("[{}]", __cv.join(",")) }; } });
    __FRACTAL_CALL_STACK.with(|__s| __s.borrow_mut().push((format!("add_edge({:?}, {:?}, {:?}, {:?})", fractal_g, fractal_u, fractal_v, fractal_w), { let __iv: Vec<String> = vec![__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_v", ":int", &{ let __v = &fractal_v; format!("{:?}", __v) }), __fractal_debug_var("fractal_w", ":int", &{ let __v = &fractal_w; format!("{:?}", __v) })]; format!("[{}]", __iv.join(",")) })));
    let mut fractal_e: Option<Box<FractalEdge>> = Some(Box::new(FractalEdge { to: Some(0), weight: Some(0) }));
    __fractal_debug_snapshot!("StructDecl e : Edge", "add_edge", 21, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_v", ":int", &{ let __v = &fractal_v; format!("{:?}", __v) }), __fractal_debug_var("fractal_w", ":int", &{ let __v = &fractal_w; format!("{:?}", __v) }), __fractal_debug_var("fractal_e", ":struct<Edge>", &{ let __v = &fractal_e; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_e.as_mut().unwrap().to = Some(fractal_v);
    __fractal_debug_snapshot!("Assign Eq", "add_edge", 22, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_v", ":int", &{ let __v = &fractal_v; format!("{:?}", __v) }), __fractal_debug_var("fractal_w", ":int", &{ let __v = &fractal_w; format!("{:?}", __v) }), __fractal_debug_var("fractal_e", ":struct<Edge>", &{ let __v = &fractal_e; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_e.as_mut().unwrap().weight = Some(fractal_w);
    __fractal_debug_snapshot!("Assign Eq", "add_edge", 23, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_v", ":int", &{ let __v = &fractal_v; format!("{:?}", __v) }), __fractal_debug_var("fractal_w", ":int", &{ let __v = &fractal_w; format!("{:?}", __v) }), __fractal_debug_var("fractal_e", ":struct<Edge>", &{ let __v = &fractal_e; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_g.as_mut().unwrap().adj.as_mut().unwrap()[(fractal_u as usize)].push(fractal_e.clone());
    __fractal_debug_snapshot!("Call append", "add_edge", 24, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_v", ":int", &{ let __v = &fractal_v; format!("{:?}", __v) }), __fractal_debug_var("fractal_w", ":int", &{ let __v = &fractal_w; format!("{:?}", __v) }), __fractal_debug_var("fractal_e", ":struct<Edge>", &{ let __v = &fractal_e; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    __FRACTAL_CALL_STACK.with(|__s| { __s.borrow_mut().pop(); });
}

pub fn fractal_dijkstra(mut fractal_g: &mut Option<Box<FractalGraph>>, mut fractal_src: i64) -> Vec<i64> {
    __fractal_debug_init();
    __FRACTAL_CALL_STACK.with(|__stk| { if let Some(__top) = __stk.borrow_mut().last_mut() { __top.1 = { let __cv: Vec<String> = vec![__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) })]; format!("[{}]", __cv.join(",")) }; } });
    __FRACTAL_CALL_STACK.with(|__s| __s.borrow_mut().push((format!("dijkstra({:?}, {:?})", fractal_g, fractal_src), { let __iv: Vec<String> = vec![__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) })]; format!("[{}]", __iv.join(",")) })));
    let mut fractal_dist: Vec<i64> = Vec::new();
    __fractal_debug_snapshot!("Decl dist : :list =", "dijkstra", 27, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_visited: Vec<bool> = Vec::new();
    __fractal_debug_snapshot!("Decl visited : :list =", "dijkstra", 28, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    {
        let mut fractal_i: i64 = 0_i64;
        __fractal_debug_snapshot!("For i", "dijkstra", 29, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        while fractal_i < fractal_g.as_ref().unwrap().n.unwrap() {
            fractal_dist.push(999999_i64.clone());
            __fractal_debug_snapshot!("Call append", "dijkstra", 30, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_visited.push(false.clone());
            __fractal_debug_snapshot!("Call append", "dijkstra", 31, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_i += 1_i64;
        }
    }
    __fractal_debug_snapshot!("For i", "dijkstra", 29, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_dist[(fractal_src as usize)] = 0_i64;
    __fractal_debug_snapshot!("Assign Eq", "dijkstra", 33, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    {
        let mut fractal_iter: i64 = 0_i64;
        __fractal_debug_snapshot!("For iter", "dijkstra", 34, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        while fractal_iter < fractal_g.as_ref().unwrap().n.unwrap() {
            let mut fractal_u: i64 = (-1_i64);
            __fractal_debug_snapshot!("Decl u : :int =", "dijkstra", 35, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            {
                let mut fractal_i: i64 = 0_i64;
                __fractal_debug_snapshot!("For i", "dijkstra", 36, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
                __fractal_debug_wait();
                while fractal_i < fractal_g.as_ref().unwrap().n.unwrap() {
                    if (!fractal_visited[fractal_i as usize]) {
                        if (fractal_u != (-1_i64)) {
                            if (fractal_dist[fractal_i as usize] < fractal_dist[fractal_u as usize]) {
                                fractal_u = fractal_i;
                            }
                        }
                        if (fractal_u == (-1_i64)) {
                            fractal_u = fractal_i;
                        }
                    }
                    fractal_i += 1_i64;
                }
            }
            __fractal_debug_snapshot!("For i", "dijkstra", 36, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            if (fractal_u == (-1_i64)) {
                break;
            }
            fractal_visited[(fractal_u as usize)] = true;
            __fractal_debug_snapshot!("Assign Eq", "dijkstra", 51, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            {
                let mut fractal_j: i64 = 0_i64;
                __fractal_debug_snapshot!("For j", "dijkstra", 52, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) })], false, None::<&str>);
                __fractal_debug_wait();
                while fractal_j < (fractal_g.as_ref().unwrap().adj.as_ref().unwrap()[fractal_u as usize].len() as i64) {
                    let mut fractal_e: Option<Box<FractalEdge>> = fractal_g.as_ref().unwrap().adj.as_ref().unwrap()[fractal_u as usize][fractal_j as usize].clone();
                    __fractal_debug_snapshot!("StructDecl e : Edge", "dijkstra", 53, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_e", ":struct<Edge>", &{ let __v = &fractal_e; format!("{:?}", __v) })], false, None::<&str>);
                    __fractal_debug_wait();
                    let mut fractal_nd: i64 = (fractal_dist[fractal_u as usize] + fractal_e.as_ref().unwrap().weight.unwrap());
                    __fractal_debug_snapshot!("Decl nd : :int =", "dijkstra", 54, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_e", ":struct<Edge>", &{ let __v = &fractal_e; format!("{:?}", __v) }), __fractal_debug_var("fractal_nd", ":int", &{ let __v = &fractal_nd; format!("{:?}", __v) })], false, None::<&str>);
                    __fractal_debug_wait();
                    if (fractal_nd < fractal_dist[fractal_e.as_ref().unwrap().to.unwrap() as usize]) {
                        fractal_dist[(fractal_e.as_ref().unwrap().to.unwrap() as usize)] = fractal_nd;
                    }
                    fractal_j += 1_i64;
                }
            }
            __fractal_debug_snapshot!("For j", "dijkstra", 52, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) }), __fractal_debug_var("fractal_iter", ":int", &{ let __v = &fractal_iter; format!("{:?}", __v) }), __fractal_debug_var("fractal_u", ":int", &{ let __v = &fractal_u; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_iter += 1_i64;
        }
    }
    __fractal_debug_snapshot!("For iter", "dijkstra", 34, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    __fractal_debug_snapshot!("Return", "dijkstra", 60, "", [__fractal_debug_var("fractal_g", ":struct<Graph>", &{ let __v = &fractal_g; format!("{:?}", __v) }), __fractal_debug_var("fractal_src", ":int", &{ let __v = &fractal_src; format!("{:?}", __v) }), __fractal_debug_var("fractal_dist", ":list", &{ let __v = &fractal_dist; format!("{:?}", __v) }), __fractal_debug_var("fractal_visited", ":list", &{ let __v = &fractal_visited; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    return fractal_dist;
    __FRACTAL_CALL_STACK.with(|__s| { __s.borrow_mut().pop(); });
}

fn main() {
    __fractal_debug_init();
    let __fractal_lock_path = std::env::var("FRACTAL_DEBUG_LOCK").unwrap_or_default();
    if !__fractal_lock_path.is_empty() { __FRACTAL_DBG_LOCK.set(__fractal_lock_path).ok(); }
    let mut fractal_graph: Option<Box<FractalGraph>> = fractal_make_graph(5_i64);
    __fractal_debug_snapshot!("StructDecl graph : Graph", "<main>", 62, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_add_edge(&mut fractal_graph, 0_i64, 1_i64, 10_i64);
    __fractal_debug_snapshot!("Call add_edge", "<main>", 63, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_add_edge(&mut fractal_graph, 0_i64, 2_i64, 3_i64);
    __fractal_debug_snapshot!("Call add_edge", "<main>", 64, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_add_edge(&mut fractal_graph, 2_i64, 1_i64, 4_i64);
    __fractal_debug_snapshot!("Call add_edge", "<main>", 65, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_add_edge(&mut fractal_graph, 1_i64, 3_i64, 2_i64);
    __fractal_debug_snapshot!("Call add_edge", "<main>", 66, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_add_edge(&mut fractal_graph, 2_i64, 3_i64, 8_i64);
    __fractal_debug_snapshot!("Call add_edge", "<main>", 67, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_add_edge(&mut fractal_graph, 3_i64, 4_i64, 5_i64);
    __fractal_debug_snapshot!("Call add_edge", "<main>", 68, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_dists: Vec<i64> = fractal_dijkstra(&mut fractal_graph, 0_i64);
    __fractal_debug_snapshot!("Decl dists : :list =", "<main>", 69, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) }), __fractal_debug_var("fractal_dists", ":list", &{ let __v = &fractal_dists; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    {
        let mut fractal_i: i64 = 0_i64;
        __fractal_debug_snapshot!("For i", "<main>", 70, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) }), __fractal_debug_var("fractal_dists", ":list", &{ let __v = &fractal_dists; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        while fractal_i < 5_i64 {
            { print!("dist[{}] = {}", fractal_i, fractal_dists[fractal_i as usize]); io::stdout().flush().unwrap(); };
            __fractal_debug_snapshot!("Call print", "<main>", 71, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) }), __fractal_debug_var("fractal_dists", ":list", &{ let __v = &fractal_dists; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_i += 1_i64;
        }
    }
    __fractal_debug_snapshot!("For i", "<main>", 70, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) }), __fractal_debug_var("fractal_dists", ":list", &{ let __v = &fractal_dists; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    __fractal_debug_snapshot!("Program finished", "<main>", 0, "", [__fractal_debug_var("fractal_graph", ":struct<Graph>", &{ let __v = &fractal_graph; format!("{:?}", __v) }), __fractal_debug_var("fractal_dists", ":list", &{ let __v = &fractal_dists; format!("{:?}", __v) })], true, None::<&str>);
}
