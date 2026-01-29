// src/runtime/host.rs
use crate::runtime::std::{std_free, std_malloc};
use libc::strlen;
use reqwest::blocking::Client;
use std::ffi::{CStr, CString, c_char};
use std::io;
use std::os::raw::c_void;
use std::ptr;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;

/// Returns the current datetime as milliseconds since UNIX epoch.
///
/// # Safety
/// No safety concerns as there are no parameters.
pub unsafe extern "C" fn host_datetime_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Frees a pointer using std free.
///
/// # Safety
/// Pointer must be valid or null, no use after free.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        std_free(ptr as *mut u8);
    }
}

/// Performs a real HTTP GET using reqwest and returns body length.
///
/// # Safety
/// The url must be a valid null-terminated C string.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_http_get(url: *const c_char) -> i64 {
    if url.is_null() {
        return -1;
    }

    let url_str = match CStr::from_ptr(url).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let client = Client::builder()
        .use_rustls_tls()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true) // for testing/demo; remove in production
        .build()
        .unwrap_or_else(|_| Client::new());

    let resp = match client.get(url_str).send() {
        Ok(r) => r,
        Err(_) => return -4,
    };

    if resp.status().is_success() {
        resp.content_length().map(|l| l as i64).unwrap_or(-2)
    } else {
        -3
    }
}

/// Performs a real TLS handshake using rustls (dummy success for now).
///
/// # Safety
/// The host must be a valid null-terminated C string.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_tls_handshake(_host: *const c_char) -> i64 {
    0
}

/// Concatenates two strings and returns new null-terminated pointer.
///
/// # Safety
/// Both inputs must be valid null-terminated string pointers (i64 cast). Caller must free returned pointer.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_concat(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    let s1 = unsafe { CStr::from_ptr(a as *const c_char) }
        .to_str()
        .unwrap_or("");
    let s2 = unsafe { CStr::from_ptr(b as *const c_char) }
        .to_str()
        .unwrap_or("");
    let concat = format!("{}{}", s1, s2);
    let cstring = CString::new(concat).unwrap();
    let len = cstring.as_bytes_with_nul().len();
    let ptr = std_malloc(len);
    unsafe {
        ptr::copy_nonoverlapping(cstring.as_ptr(), ptr as *mut c_char, len);
    }
    ptr as i64
}

/// Returns string length.
///
/// # Safety
/// Input must be valid null-terminated string pointer (i64 cast).
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_len(s: i64) -> i64 {
    if s == 0 {
        0
    } else {
        strlen(s as *const c_char) as i64
    }
}

/// Converts string to lowercase and returns new pointer.
///
/// # Safety
/// Input must be valid null-terminated string pointer. Caller must free returned pointer.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_to_lowercase(s: i64) -> i64 {
    string_op(s, |st| st.to_lowercase())
}

/// Converts string to uppercase and returns new pointer.
///
/// # Safety
/// Input must be valid null-terminated string pointer. Caller must free returned pointer.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_to_uppercase(s: i64) -> i64 {
    string_op(s, |st| st.to_uppercase())
}

/// Trims string and returns new pointer.
///
/// # Safety
/// Input must be valid null-terminated string pointer. Caller must free returned pointer.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_trim(s: i64) -> i64 {
    string_op(s, |st| st.trim().to_string())
}

/// Checks if string starts with substring, returns 1 if true, 0 otherwise.
///
/// # Safety
/// Both inputs must be valid null-terminated string pointers (i64 cast).
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_starts_with(haystack: i64, needle: i64) -> i64 {
    string_pred(haystack, needle, |h, n| h.starts_with(n))
}

/// Checks if string ends with substring, returns 1 if true, 0 otherwise.
///
/// # Safety
/// Both inputs must be valid null-terminated string pointers (i64 cast).
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_ends_with(haystack: i64, needle: i64) -> i64 {
    string_pred(haystack, needle, |h, n| h.ends_with(n))
}

/// Checks if string contains substring, returns 1 if true, 0 otherwise.
///
/// # Safety
/// Both inputs must be valid null-terminated string pointers (i64 cast).
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_contains(haystack: i64, needle: i64) -> i64 {
    string_pred(haystack, needle, |h, n| h.contains(n))
}

/// Replaces all occurrences of old with new and returns new null-terminated pointer.
///
/// # Safety
/// All three inputs must be valid null-terminated string pointers (i64 cast). Caller must free returned pointer.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_str_replace(s: i64, old: i64, new: i64) -> i64 {
    if s == 0 || old == 0 || new == 0 {
        return 0;
    }
    let str_val = unsafe { CStr::from_ptr(s as *const c_char) }
        .to_str()
        .unwrap_or("");
    let old_val = unsafe { CStr::from_ptr(old as *const c_char) }
        .to_str()
        .unwrap_or("");
    let new_val = unsafe { CStr::from_ptr(new as *const c_char) }
        .to_str()
        .unwrap_or("");
    let replaced = str_val.replace(old_val, new_val);
    let cstring = CString::new(replaced).unwrap();
    let len = cstring.as_bytes_with_nul().len();
    let ptr = std_malloc(len);
    unsafe {
        ptr::copy_nonoverlapping(cstring.as_ptr(), ptr as *mut c_char, len);
    }
    ptr as i64
}

/// Prints a null-terminated string with newline.
///
/// # Safety
/// Input must be a valid null-terminated string pointer (i64 cast).
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_println(s: i64) -> i64 {
    if s == 0 {
        println!();
        return 0;
    }
    let st = unsafe { CStr::from_ptr(s as *const c_char) }
        .to_str()
        .unwrap_or("");
    println!("{}", st);
    0
}

/// Reads a line from stdin and returns a null-terminated string pointer.
///
/// # Safety
/// Caller must free returned pointer. Returns 0 on EOF or error.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_read_line() -> i64 {
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return 0;
    }
    if input.is_empty() {
        return 0;
    }
    // Normalize line endings and avoid interior nulls.
    let input = input.trim_end_matches(&['\n', '\r'][..]).replace('\0', "");
    let cstring = CString::new(input).unwrap();
    let len = cstring.as_bytes_with_nul().len();
    let ptr = std_malloc(len);
    unsafe {
        ptr::copy_nonoverlapping(cstring.as_ptr(), ptr as *mut c_char, len);
    }
    ptr as i64
}

/// Reads a line from stdin and parses it as i64.
///
/// # Safety
/// Returns 0 on EOF or parse error.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_read_int() -> i64 {
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return 0;
    }
    input.trim().parse::<i64>().unwrap_or(0)
}

#[derive(Clone)]
struct BjCard {
    name: String,
    value: i64,
    is_ace: bool,
}

struct BlackjackState {
    deck: Vec<BjCard>,
    idx: usize,
    last: Option<BjCard>,
    rng: u64,
}

impl BlackjackState {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self {
            deck: Vec::new(),
            idx: 0,
            last: None,
            rng: seed ^ 0x9E3779B97F4A7C15,
        }
    }

    fn next_rand(&mut self) -> u64 {
        // simple LCG
        self.rng = self.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.rng
    }

    fn reset(&mut self) {
        self.deck = build_bj_deck();
        // Fisher-Yates shuffle
        let len = self.deck.len();
        let mut i = len;
        while i > 1 {
            i -= 1;
            let j = (self.next_rand() % (i as u64 + 1)) as usize;
            self.deck.swap(i, j);
        }
        self.idx = 0;
        self.last = None;
    }

    fn draw(&mut self) -> Option<BjCard> {
        if self.idx >= self.deck.len() {
            return None;
        }
        let card = self.deck[self.idx].clone();
        self.idx += 1;
        self.last = Some(card.clone());
        Some(card)
    }
}

fn build_bj_deck() -> Vec<BjCard> {
    let ranks: [(&str, i64, bool); 13] = [
        ("A", 11, true),
        ("2", 2, false),
        ("3", 3, false),
        ("4", 4, false),
        ("5", 5, false),
        ("6", 6, false),
        ("7", 7, false),
        ("8", 8, false),
        ("9", 9, false),
        ("10", 10, false),
        ("J", 10, false),
        ("Q", 10, false),
        ("K", 10, false),
    ];
    let suits = ["S", "H", "D", "C"];
    let mut deck = Vec::with_capacity(52);
    for suit in suits {
        for (rank, value, is_ace) in ranks {
            deck.push(BjCard {
                name: format!("{}{}", rank, suit),
                value,
                is_ace,
            });
        }
    }
    deck
}

lazy_static! {
    static ref BJ_STATE: Mutex<BlackjackState> = Mutex::new(BlackjackState::new());
}

fn alloc_cstring(s: &str) -> i64 {
    let cstring = CString::new(s).unwrap();
    let len = cstring.as_bytes_with_nul().len();
    let ptr = unsafe { std_malloc(len) };
    unsafe {
        ptr::copy_nonoverlapping(cstring.as_ptr(), ptr as *mut c_char, len);
    }
    ptr as i64
}

/// Initializes and shuffles a 52-card deck for blackjack.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_bj_init() -> i64 {
    if let Ok(mut state) = BJ_STATE.lock() {
        state.reset();
        return 1;
    }
    0
}

/// Draws the next card. Returns its value (Ace=11).
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_bj_draw() -> i64 {
    if let Ok(mut state) = BJ_STATE.lock() {
        if let Some(card) = state.draw() {
            return card.value;
        }
    }
    0
}

/// Returns the name of the last drawn card (e.g., "AS").
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_bj_last_name() -> i64 {
    if let Ok(state) = BJ_STATE.lock() {
        if let Some(card) = &state.last {
            return alloc_cstring(&card.name);
        }
    }
    0
}

/// Returns 1 if the last drawn card was an Ace, else 0.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn host_bj_last_is_ace() -> i64 {
    if let Ok(state) = BJ_STATE.lock() {
        if let Some(card) = &state.last {
            return if card.is_ace { 1 } else { 0 };
        }
    }
    0
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn string_op<F>(s: i64, op: F) -> i64
where
    F: FnOnce(&str) -> String,
{
    if s == 0 {
        return 0;
    }
    let input = unsafe { CStr::from_ptr(s as *const c_char) }
        .to_str()
        .unwrap_or("");
    let result = op(input);
    let cstring = CString::new(result).unwrap();
    let len = cstring.as_bytes_with_nul().len();
    let ptr = std_malloc(len);
    unsafe {
        ptr::copy_nonoverlapping(cstring.as_ptr(), ptr as *mut c_char, len);
    }
    ptr as i64
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn string_pred<F>(haystack: i64, needle: i64, pred: F) -> i64
where
    F: FnOnce(&str, &str) -> bool,
{
    if haystack == 0 || needle == 0 {
        return 0;
    }
    let hay = unsafe { CStr::from_ptr(haystack as *const c_char) }
        .to_str()
        .unwrap_or("");
    let ndl = unsafe { CStr::from_ptr(needle as *const c_char) }
        .to_str()
        .unwrap_or("");
    if pred(hay, ndl) { 1 } else { 0 }
}
