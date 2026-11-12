//! Which graphics API draws the wallpaper, how much memory the process keeps, and how much CPU it really uses.
//!
//! GPUI tries Vulkan first and OpenGL when Vulkan fails. On Intel Ivy Bridge and Haswell, Mesa's Vulkan driver
//! (hasvk) opens but warns "Haswell Vulkan support is incomplete", and it can stall or draw wrong; Mesa's OpenGL
//! driver for the same chips (crocus) is complete. So there Vulkan is hidden from this process before GPUI starts,
//! and GPUI falls back to OpenGL by itself. `--gpu vulkan|gl|auto` or PULSE_GPU decide it by hand.

/// Must run first in main(), before any thread exists (it sets environment variables).
pub fn choose(arg: Option<&str>) {
    let pick = arg.map(str::to_string).or_else(|| std::env::var("PULSE_GPU").ok()).unwrap_or_default().to_ascii_lowercase();
    let gl = match pick.as_str() {
        "gl" | "opengl" => true,
        "vulkan" | "vk" => false,
        _ => old_intel().is_some(),
    };
    if gl {
        // the Vulkan loader then finds no driver at all, Vulkan fails at once and GPUI takes OpenGL
        std::env::set_var("VK_DRIVER_FILES", "/nonexistent/pulse-native-no-vulkan.json");
        std::env::set_var("VK_ICD_FILENAMES", "/nonexistent/pulse-native-no-vulkan.json");
        if crate::perf::on() || std::env::var("PULSE_GPU_LOG").is_ok() {
            eprintln!("pulse-native: drawing with OpenGL{}", old_intel().map(|g| format!(" ({g}: Mesa's Vulkan is incomplete there)")).unwrap_or_default());
        }
    }
    trim_setup();
}

/// An Intel GPU of generation 7 or 7.5 (Ivy Bridge, Bay Trail, Haswell): its name when there is one.
fn old_intel() -> Option<&'static str> {
    let dir = std::fs::read_dir("/sys/class/drm").ok()?;
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let read = |f: &str| std::fs::read_to_string(e.path().join("device").join(f)).ok().and_then(|s| u32::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok());
        if read("vendor") != Some(0x8086) {
            continue;
        }
        let Some(id) = read("device") else { continue };
        if let Some(g) = gen7(id) {
            return Some(g);
        }
    }
    None
}

/// Mesa's PCI ids for these chips (include/pci_ids/i965_pci_ids.h).
fn gen7(id: u32) -> Option<&'static str> {
    match id {
        0x0152 | 0x0156 | 0x015a | 0x0162 | 0x0166 | 0x016a => Some("Ivy Bridge"),
        0x0f31 | 0x0f32 | 0x0f33 | 0x0157 | 0x0155 => Some("Bay Trail"),
        0x0402..=0x042e | 0x0a02..=0x0a2e | 0x0b22 | 0x0c02..=0x0c2e | 0x0d02..=0x0d2e => Some("Haswell"),
        _ => None,
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
extern "C" {
    fn mallopt(param: i32, value: i32) -> i32;
    fn malloc_trim(pad: usize) -> i32;
}

/// glibc keeps freed memory to reuse it, and raises the size it gives back to the system each time a big buffer is
/// freed: the sphere's two pictures (a few hundred kB, new every frame) then stay in the heap and the process grows
/// slowly. A fixed threshold sends them back at once; two arenas instead of one per thread keep less memory aside.
fn trim_setup() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    unsafe {
        const M_MMAP_THRESHOLD: i32 = -3;
        const M_ARENA_MAX: i32 = -8;
        mallopt(M_MMAP_THRESHOLD, 128 * 1024);
        mallopt(M_ARENA_MAX, 2);
    }
}

/// Gives the memory freed since the last time back to the system (once a minute is enough).
pub fn trim() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    unsafe {
        malloc_trim(0);
    }
}

/// This whole process's CPU time in seconds (every thread: drawing, the GPU driver, the hub reader).
pub fn cpu_seconds() -> Option<f64> {
    let s = std::fs::read_to_string("/proc/self/stat").ok()?;
    let rest = &s[s.rfind(')')? + 2..];
    let f: Vec<&str> = rest.split_whitespace().collect();
    let (u, k) = (f.get(11)?.parse::<f64>().ok()?, f.get(12)?.parse::<f64>().ok()?);
    Some((u + k) / 100.0) // USER_HZ is 100 on Linux
}
