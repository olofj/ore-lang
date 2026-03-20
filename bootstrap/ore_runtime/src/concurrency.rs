use std::sync::Mutex;

// ── Thread handles ──

static THREADS: Mutex<Vec<std::thread::JoinHandle<()>>> = Mutex::new(Vec::new());

fn register_thread(handle: std::thread::JoinHandle<()>) {
    THREADS.lock().unwrap().push(handle);
}

// ── Spawn ──

#[no_mangle]
pub extern "C" fn ore_spawn(func: extern "C" fn()) {
    register_thread(std::thread::spawn(move || func()));
}

/// Spawn a function that takes a single i64 argument (used for passing channels, etc.)
#[no_mangle]
pub extern "C" fn ore_spawn_with_arg(func: extern "C" fn(i64), arg: i64) {
    register_thread(std::thread::spawn(move || func(arg)));
}

#[no_mangle]
pub extern "C" fn ore_spawn_with_2args(func: extern "C" fn(i64, i64), a: i64, b: i64) {
    register_thread(std::thread::spawn(move || func(a, b)));
}

#[no_mangle]
pub extern "C" fn ore_spawn_with_3args(func: extern "C" fn(i64, i64, i64), a: i64, b: i64, c: i64) {
    register_thread(std::thread::spawn(move || func(a, b, c)));
}

#[no_mangle]
pub extern "C" fn ore_thread_join_all() {
    let mut threads = THREADS.lock().unwrap();
    for handle in threads.drain(..) {
        handle.join().unwrap();
    }
}

#[no_mangle]
pub extern "C" fn ore_sleep(ms: i64) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

// ── Channels ──

use std::sync::mpsc;

pub struct OreChannel {
    sender: Mutex<mpsc::Sender<i64>>,
    receiver: Mutex<mpsc::Receiver<i64>>,
}

#[no_mangle]
pub extern "C" fn ore_channel_new() -> *mut OreChannel {
    let (tx, rx) = mpsc::channel();
    let ch = Box::new(OreChannel {
        sender: Mutex::new(tx),
        receiver: Mutex::new(rx),
    });
    Box::into_raw(ch)
}

#[no_mangle]
pub extern "C" fn ore_channel_send(ch: *mut OreChannel, val: i64) {
    let ch = unsafe { &*ch };
    ch.sender.lock().unwrap().send(val).unwrap();
}

#[no_mangle]
pub extern "C" fn ore_channel_recv(ch: *mut OreChannel) -> i64 {
    let ch = unsafe { &*ch };
    ch.receiver.lock().unwrap().recv().unwrap()
}
