#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

use std::sync::{Mutex, Once};
use std::fs::{OpenOptions, File as __DbgFile};
use std::io::{BufWriter as __DbgBufWriter, Write as __DbgWrite};

static __FRACTAL_DBG_INIT: Once = Once::new();
#[allow(clippy::type_complexity)]
static __FRACTAL_DBG_FILE: Mutex<Option<__DbgBufWriter<__DbgFile>>> = Mutex::new(None);
static __FRACTAL_DBG_PREV: std::sync::OnceLock<Mutex<std::collections::HashMap<String, String>>> = std::sync::OnceLock::new();
static __FRACTAL_DBG_STEP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn __fractal_debug_init() {
    __FRACTAL_DBG_INIT.call_once(|| {
        let __f = OpenOptions::new().create(true).write(true).truncate(true).open("/home/theerttha/code/Fractal/jnlml.debug.jsonl").expect("cannot open fractal debug file");
        *__FRACTAL_DBG_FILE.lock().unwrap() = Some(__DbgBufWriter::new(__f));
    });
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

fn main() {}
