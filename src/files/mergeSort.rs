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

fn __fractal_debug_init() {
    __FRACTAL_DBG_INIT.call_once(|| {
        let __f = OpenOptions::new().create(true).write(true).truncate(true).open("/home/theerttha/code/Fractal/src/files/mergeSort.debug.jsonl").expect("cannot open fractal debug file");
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
    ($label:expr, $func:expr, $line:expr, [$($var_str:expr),* $(,)?], $finished:expr, $error:expr) => {{
        let __dbg_step = __FRACTAL_DBG_STEP.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut __dbg_g = __FRACTAL_DBG_FILE.lock().unwrap();
        if let Some(ref mut __dbg_w) = *__dbg_g {
            let __dbg_vars: Vec<String> = vec![$($var_str),*];
            let __dbg_scope = {
                let mut __sc = String::from("{\"label\":\"");
                __sc.push_str(&__fractal_debug_json_escape($func));
                __sc.push_str("\",\"vars\":[");
                __sc.push_str(&__dbg_vars.join(","));
                __sc.push_str("]}");
                __sc
            };
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
                __ln.push_str(",\"stack\":[\"");
                __ln.push_str(&__fractal_debug_json_escape($func));
                __ln.push_str("\"],\"scopes\":[");
                __ln.push_str(&__dbg_scope);
                __ln.push_str("],\"output\":\"\",\"finished\":");
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

pub fn fractal_merge(mut fractal_arr: &mut Vec<i64>, mut fractal_l: i64, mut fractal_m: i64, mut fractal_r: i64) {
    __fractal_debug_init();
    let mut fractal_n1: i64 = ((fractal_m - fractal_l) + 1_i64);
    __fractal_debug_snapshot!("Decl n1 : :int =", "merge", 7, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_n2: i64 = (fractal_r - fractal_m);
    __fractal_debug_snapshot!("Decl n2 : :int =", "merge", 8, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_L: Vec<i64> = Vec::new();
    __fractal_debug_snapshot!("Decl L : :list =", "merge", 10, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_R: Vec<i64> = Vec::new();
    __fractal_debug_snapshot!("Decl R : :list =", "merge", 11, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    {
        let mut fractal_i: i64 = 0_i64;
        while fractal_i < fractal_n1 {
            __fractal_debug_snapshot!("For i", "merge", 13, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_L.push(fractal_arr[(fractal_l + fractal_i) as usize].clone());
            __fractal_debug_snapshot!("ExprStmt", "merge", 14, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_i += 1_i64;
        }
    }
    __fractal_debug_snapshot!("For i", "merge", 13, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    {
        let mut fractal_j: i64 = 0_i64;
        while fractal_j < fractal_n2 {
            __fractal_debug_snapshot!("For j", "merge", 17, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_R.push(fractal_arr[((fractal_m + 1_i64) + fractal_j) as usize].clone());
            __fractal_debug_snapshot!("ExprStmt", "merge", 18, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_j += 1_i64;
        }
    }
    __fractal_debug_snapshot!("For j", "merge", 17, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_i: i64 = 0_i64;
    __fractal_debug_snapshot!("Decl i : :int =", "merge", 21, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_j: i64 = 0_i64;
    __fractal_debug_snapshot!("Decl j : :int =", "merge", 22, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_k: i64 = fractal_l;
    __fractal_debug_snapshot!("Decl k : :int =", "merge", 23, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    __fractal_debug_snapshot!("While", "merge", 25, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    while ((fractal_i < fractal_n1) && (fractal_j < fractal_n2)) {
        __fractal_debug_snapshot!("If", "merge", 26, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        if (fractal_L[fractal_i as usize] <= fractal_R[fractal_j as usize]) {
            let __idx_0 = fractal_k as usize;
            fractal_arr[__idx_0] = fractal_L[fractal_i as usize];
            __fractal_debug_snapshot!("Assign Eq", "merge", 27, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_i += 1_i64;
            __fractal_debug_snapshot!("Assign PlusEq", "merge", 28, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
        } else {
            let __idx_1 = fractal_k as usize;
            fractal_arr[__idx_1] = fractal_R[fractal_j as usize];
            __fractal_debug_snapshot!("Assign Eq", "merge", 31, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_j += 1_i64;
            __fractal_debug_snapshot!("Assign PlusEq", "merge", 32, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
        }
        fractal_k += 1_i64;
        __fractal_debug_snapshot!("Assign PlusEq", "merge", 34, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
    }
    __fractal_debug_snapshot!("While", "merge", 37, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    while (fractal_i < fractal_n1) {
        let __idx_2 = fractal_k as usize;
        fractal_arr[__idx_2] = fractal_L[fractal_i as usize];
        __fractal_debug_snapshot!("Assign Eq", "merge", 38, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        fractal_i += 1_i64;
        __fractal_debug_snapshot!("Assign PlusEq", "merge", 39, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        fractal_k += 1_i64;
        __fractal_debug_snapshot!("Assign PlusEq", "merge", 40, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
    }
    __fractal_debug_snapshot!("While", "merge", 43, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    while (fractal_j < fractal_n2) {
        let __idx_3 = fractal_k as usize;
        fractal_arr[__idx_3] = fractal_R[fractal_j as usize];
        __fractal_debug_snapshot!("Assign Eq", "merge", 44, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        fractal_j += 1_i64;
        __fractal_debug_snapshot!("Assign PlusEq", "merge", 45, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        fractal_k += 1_i64;
        __fractal_debug_snapshot!("Assign PlusEq", "merge", 46, [__fractal_debug_var("fractal_n1", ":int", &{ let __v = &fractal_n1; format!("{:?}", __v) }), __fractal_debug_var("fractal_n2", ":int", &{ let __v = &fractal_n2; format!("{:?}", __v) }), __fractal_debug_var("fractal_L", ":list", &{ let __v = &fractal_L; format!("{:?}", __v) }), __fractal_debug_var("fractal_R", ":list", &{ let __v = &fractal_R; format!("{:?}", __v) }), __fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) }), __fractal_debug_var("fractal_j", ":int", &{ let __v = &fractal_j; format!("{:?}", __v) }), __fractal_debug_var("fractal_k", ":int", &{ let __v = &fractal_k; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
    }
}

pub fn fractal_mergeSort(mut fractal_arr: &mut Vec<i64>, mut fractal_l: i64, mut fractal_r: i64) {
    __fractal_debug_init();
    __fractal_debug_snapshot!("If", "mergeSort", 51, [], false, None::<&str>);
    __fractal_debug_wait();
    if (fractal_l < fractal_r) {
        let mut fractal_m: i64 = ((fractal_l + fractal_r) / 2_i64);
        __fractal_debug_snapshot!("Decl m : :int =", "mergeSort", 52, [__fractal_debug_var("fractal_m", ":int", &{ let __v = &fractal_m; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        fractal_mergeSort(fractal_arr, fractal_l, fractal_m);
        __fractal_debug_snapshot!("ExprStmt", "mergeSort", 53, [__fractal_debug_var("fractal_m", ":int", &{ let __v = &fractal_m; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        fractal_mergeSort(fractal_arr, (fractal_m + 1_i64), fractal_r);
        __fractal_debug_snapshot!("ExprStmt", "mergeSort", 54, [__fractal_debug_var("fractal_m", ":int", &{ let __v = &fractal_m; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        fractal_merge(fractal_arr, fractal_l, fractal_m, fractal_r);
        __fractal_debug_snapshot!("ExprStmt", "mergeSort", 55, [__fractal_debug_var("fractal_m", ":int", &{ let __v = &fractal_m; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
    }
}

fn main() {
    __fractal_debug_init();
    let __fractal_lock_path = std::env::var("FRACTAL_DEBUG_LOCK").unwrap_or_default();
    if !__fractal_lock_path.is_empty() { __FRACTAL_DBG_LOCK.set(__fractal_lock_path).ok(); }
    let mut fractal_n: i64 = 5_i64;
    __fractal_debug_snapshot!("Decl n : :int =", "<main>", 3, [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    let mut fractal_a: Vec<i64> = vec![5_i64, 4_i64, 3_i64, 2_i64, 1_i64];
    __fractal_debug_snapshot!("Decl a : :list =", "<main>", 4, [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_a", ":list", &{ let __v = &fractal_a; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    fractal_mergeSort(unsafe { &mut *(&mut fractal_a as *mut _) }, 0_i64, (fractal_n - 1_i64));
    __fractal_debug_snapshot!("ExprStmt", "<main>", 59, [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_a", ":list", &{ let __v = &fractal_a; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    {
        let mut fractal_i: i64 = 0_i64;
        while fractal_i < fractal_n {
            __fractal_debug_snapshot!("For i", "<main>", 60, [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_a", ":list", &{ let __v = &fractal_a; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            { print!("{}\t", fractal_a[fractal_i as usize]); io::stdout().flush().unwrap(); };
            __fractal_debug_snapshot!("ExprStmt", "<main>", 61, [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_a", ":list", &{ let __v = &fractal_a; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_i += 1_i64;
        }
    }
    __fractal_debug_snapshot!("For i", "<main>", 60, [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_a", ":list", &{ let __v = &fractal_a; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    __fractal_debug_snapshot!("Program finished", "<main>", 0, [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) }), __fractal_debug_var("fractal_a", ":list", &{ let __v = &fractal_a; format!("{:?}", __v) })], true, None::<&str>);
}
