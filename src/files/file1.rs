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
        let __f = OpenOptions::new().create(true).write(true).truncate(true).open("/home/harikrishnanr/Code/Fractal/src/files/file1.debug.jsonl").expect("cannot open fractal debug file");
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

pub fn fractal_fibonacci(mut fractal_n: i64) -> i64 {
    __fractal_debug_init();
    __FRACTAL_CALL_STACK.with(|__stk| { if let Some(__top) = __stk.borrow_mut().last_mut() { __top.1 = { let __cv: Vec<String> = vec![__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) })]; format!("[{}]", __cv.join(",")) }; } });
    __FRACTAL_CALL_STACK.with(|__s| __s.borrow_mut().push((format!("fibonacci({:?})", fractal_n), { let __iv: Vec<String> = vec![__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) })]; format!("[{}]", __iv.join(",")) })));
    __fractal_debug_snapshot!("If", "fibonacci", 4, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) })], false, None::<&str>);
    __fractal_debug_wait();
    if (fractal_n <= 1_i64) {
        __fractal_debug_snapshot!("Return", "fibonacci", 5, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        return fractal_n;
    } else {
        __fractal_debug_snapshot!("Return", "fibonacci", 8, "", [__fractal_debug_var("fractal_n", ":int", &{ let __v = &fractal_n; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        return (fractal_fibonacci((fractal_n - 1_i64)) + fractal_fibonacci((fractal_n - 2_i64)));
    }
    __FRACTAL_CALL_STACK.with(|__s| { __s.borrow_mut().pop(); });
}

pub fn fractal_main() {
    __fractal_debug_init();
    __FRACTAL_CALL_STACK.with(|__stk| { if let Some(__top) = __stk.borrow_mut().last_mut() { __top.1 = "[]".to_string(); } });
    __FRACTAL_CALL_STACK.with(|__s| __s.borrow_mut().push(("main".to_string(), "[]".to_string())));
    { print!("Fibonacci series up to 10:\n"); io::stdout().flush().unwrap(); };
    __fractal_debug_snapshot!("Call print", "main", 13, "", [], false, None::<&str>);
    __fractal_debug_wait();
    {
        let mut fractal_i: i64 = 0_i64;
        __fractal_debug_snapshot!("For i", "main", 14, "", [__fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
        __fractal_debug_wait();
        while fractal_i < 10_i64 {
            { print!("{}", fractal_fibonacci(fractal_i)); io::stdout().flush().unwrap(); };
            __fractal_debug_snapshot!("Call print", "main", 15, "", [__fractal_debug_var("fractal_i", ":int", &{ let __v = &fractal_i; format!("{:?}", __v) })], false, None::<&str>);
            __fractal_debug_wait();
            fractal_i += 1_i64;
        }
    }
    __fractal_debug_snapshot!("For i", "main", 14, "", [], false, None::<&str>);
    __fractal_debug_wait();
    __FRACTAL_CALL_STACK.with(|__s| { __s.borrow_mut().pop(); });
}

fn main() {
    __fractal_debug_init();
    let __fractal_lock_path = std::env::var("FRACTAL_DEBUG_LOCK").unwrap_or_default();
    if !__fractal_lock_path.is_empty() { __FRACTAL_DBG_LOCK.set(__fractal_lock_path).ok(); }
    fractal_main();
    __fractal_debug_snapshot!("Call main", "<main>", 18, "", [], false, None::<&str>);
    __fractal_debug_wait();
    __fractal_debug_snapshot!("Program finished", "<main>", 0, "", [], true, None::<&str>);
}
