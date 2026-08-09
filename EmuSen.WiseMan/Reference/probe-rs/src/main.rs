// The reference probe's policy layer: arguments, the press script, the state
// anchor, the report loop and the dump format. It names no emulator - every
// machine-specific act goes through the ProbeBackend trait.
// See EmuSen_Debugging_Tools_Reference_v5.md §3.50.
mod audio;
mod backend;
mod backends;
mod dump;

use backend::{
    InputSchedule, MemorySpace, ProbeBackend, ProbeButton, ProbeIdentity, ProbeOptions, TraceKind,
    find_space,
};
use audio::AudioFormat;
use dump::{BlobCache, SignatureWriter};

const NO_FRAME: u32 = 0xFFFF_FFFF;

fn usage() {
    println!("usage: probe <rom> <outDir> <startFrame> <endFrame> [stride] [anchorAddr] [traceUntilFrame] [flags]");
    println!("  --backend NAME   which reference emulator to drive (default: {})", backends::DEFAULT);
    for line in backends::usage_lines() {
        println!("{line}");
    }
    println!("  Writes <backend>_<space>_f<frame>.bin for every memory space the backend");
    println!("  exposes, plus <backend>_screen_f<frame>.bin and a manifest naming all of");
    println!("  them with sizes and the screen's pixel format.");
    println!("  --press F:BTN[:DUR]  holds BTN on port 1 from frame F for DUR frames");
    println!("                       (default 4), repeatable - needed to reach any scene");
    println!("                       behind a menu. A/B/X/Y/L/R/Up/Down/Left/Right/Start/Select");
    println!("  --pressuntil BTN:ADDR:VALUE[:CAP[:EVERY]]  taps BTN until the word at ADDR");
    println!("                       in the backend's anchor space equals VALUE (all hex).");
    println!("                       Anchor on a value both emulators already agree about.");
    println!("  --after N        report for N frames past a --pressuntil anchor, no trace");
    println!("  --cputrace F     record every CPU instruction from boot to frame F");
    println!("  --apulog F       record every $4000-$4017 write from boot to frame F");
    println!("  --audio PATH     record mixed audio, FLAC unless PATH says .wav");
    println!("  --wav PATH       record mixed audio uncompressed - the fallback, and");
    println!("                       what deep troubleshooting wants; also what a failed");
    println!("                       or unverifiable encode leaves behind");
    println!("  --ramstate zeros|ones|random   power-on RAM fill (default zeros)");
    println!("  --sig            also write <backend>_sig.csv: one row of CRCs per frame");
    println!("                       from frame 0, which is what locates a divergence");
    println!("                       rather than measuring one at the end - see §3.48");
}

// C's atoi and strtoul(.., 16): a leading integer or nothing, never an error.
// The C++ probe leaned on that for every numeric argument, so a port that
// rejected "12abc" would refuse command lines that used to run.
fn atoi(text: &str) -> u32 {
    let text = text.trim_start();
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let value: i64 = digits
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    if negative { (-value) as u32 } else { value as u32 }
}

fn strtoul_hex(text: &str) -> u32 {
    let text = text.trim_start();
    let text = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")).unwrap_or(text);
    u32::from_str_radix(
        &text.chars().take_while(char::is_ascii_hexdigit).collect::<String>(),
        16,
    )
    .unwrap_or(0)
}

// Reads the little-endian word the anchor watches, wrapping the way the space
// itself does so an address past the end is a mirror rather than a crash.
fn anchor_word(space: &MemorySpace<'_>, addr: u32) -> u32 {
    let size = space.data.len();
    if size == 0 {
        return 0;
    }
    let mask = size - 1;
    let low = space.data[addr as usize & mask] as u32;
    let high = space.data[addr.wrapping_add(1) as usize & mask] as u32;
    high << 8 | low
}

// Every frame gets a row, because a divergence hides anywhere inside a stride
// and the whole point of the stream is finding the first one - see §3.48.
fn advance_to(backend: &mut dyn ProbeBackend, signature: Option<&mut SignatureWriter>, target: u32) {
    let Some(signature) = signature else {
        backend.run_until(target);
        return;
    };

    while backend.frame_count() < target {
        let next = backend.frame_count() + 1;
        backend.run_until(next);

        let spaces = backend.spaces();
        let screen = backend.screen();
        if let Err(error) = signature.write_row(backend.frame_count(), &spaces, screen.as_ref()) {
            println!("[WARN] could not write a signature row: {error}");
        }
    }
}

struct Args {
    rom_path: String,
    dump_dir: String,
    start_frame: u32,
    end_frame: u32,
    stride: u32,
    anchor_addr: u32,
    trace_frame: u32,
    backend_name: String,
    core_path: String,
    audio_path: String,
    options: ProbeOptions,
    schedule: InputSchedule,
    cpu_trace_frame: u32,
    apu_log_frame: u32,
    after_frames: u32,
    want_signature: bool,
    audio_format: AudioFormat,
    until: Option<PressUntil>,
}

struct PressUntil {
    button: ProbeButton,
    addr: u32,
    value: u32,
    cap: u32,
    every: u32,
}

fn parse_args(argv: &[String]) -> Result<Args, i32> {
    if argv.len() < 5 {
        usage();
        return Err(1);
    }

    let dump_dir = argv[2].clone();
    let mut args = Args {
        rom_path: argv[1].clone(),
        start_frame: atoi(&argv[3]),
        end_frame: atoi(&argv[4]),
        // The first flag ends the run of optional positionals; see positional().
        stride: positional(argv, 5).map_or(1, atoi),
        anchor_addr: positional(argv, 6).map_or(0x0A00, strtoul_hex),
        trace_frame: positional(argv, 7).map_or(NO_FRAME, atoi),
        backend_name: backends::DEFAULT.to_string(),
        core_path: String::new(),
        audio_path: String::new(),
        // home_folder is filled in once the backend exists - see home_folder().
        options: ProbeOptions::default(),
        dump_dir,
        schedule: InputSchedule::default(),
        cpu_trace_frame: NO_FRAME,
        apu_log_frame: NO_FRAME,
        after_frames: 0,
        want_signature: false,
        audio_format: AudioFormat::Wav,
        until: None,
    };

    // Unknown arguments are skipped rather than rejected, matching the C++: the
    // positionals are re-walked by this same loop and must not trip it.
    let mut i = 5;
    while i < argv.len() {
        let flag = argv[i].as_str();
        let mut value = || {
            i += 1;
            argv.get(i).cloned()
        };

        match flag {
            "--sig" => args.want_signature = true,
            "--backend" => {
                if let Some(v) = value() {
                    args.backend_name = v;
                }
            }
            "--core" => {
                if let Some(v) = value() {
                    args.core_path = v;
                }
            }
            "--after" => {
                if let Some(v) = value() {
                    args.after_frames = atoi(&v);
                }
            }
            "--cputrace" => {
                if let Some(v) = value() {
                    args.cpu_trace_frame = atoi(&v);
                }
            }
            "--apulog" => {
                if let Some(v) = value() {
                    args.apu_log_frame = atoi(&v);
                }
            }
            "--wav" => {
                if let Some(v) = value() {
                    args.options.wav_path = v;
                    args.audio_format = AudioFormat::Wav;
                }
            }
            "--audio" => {
                if let Some(v) = value() {
                    args.audio_format = AudioFormat::from_path(&v);
                    // The backend can only record WAV, so it is pointed at a
                    // capture file and the policy layer re-encodes afterwards.
                    args.options.wav_path =
                        audio::capture_path(&v, args.audio_format).to_string_lossy().into_owned();
                    args.audio_path = v;
                }
            }
            "--ramstate" => {
                if let Some(v) = value() {
                    args.options.ram_state = v;
                }
            }
            "--pressuntil" => {
                if let Some(v) = value() {
                    let fields: Vec<&str> = v.split(':').collect();
                    if fields.len() < 3 {
                        println!("[ERROR] --pressuntil wants BTN:ADDR:VALUE[:CAP[:EVERY]]");
                        return Err(1);
                    }
                    let Some(button) = ProbeButton::from_name(fields[0]) else {
                        println!("[ERROR] unknown button '{}'", fields[0]);
                        return Err(1);
                    };
                    args.until = Some(PressUntil {
                        button,
                        addr: strtoul_hex(fields[1]),
                        value: strtoul_hex(fields[2]),
                        cap: fields.get(3).map_or(6000, |f| atoi(f)),
                        every: fields.get(4).map_or(40, |f| atoi(f)),
                    });
                }
            }
            "--press" => {
                if let Some(v) = value() {
                    let fields: Vec<&str> = v.split(':').collect();
                    if fields.len() < 2 {
                        println!("[ERROR] --press wants F:BTN[:DUR]");
                        return Err(1);
                    }
                    let Some(button) = ProbeButton::from_name(fields[1]) else {
                        println!("[ERROR] unknown button '{}'", fields[1]);
                        return Err(1);
                    };
                    let start = atoi(fields[0]);
                    let duration = fields.get(2).map_or(4, |f| atoi(f));
                    args.schedule.add(start, duration, button);
                    println!(
                        "[INFO] press {} frames {}..{}",
                        fields[1],
                        start,
                        start + duration - 1
                    );
                }
            }
            _ => {}
        }
        i += 1;
    }

    Ok(args)
}

// Every slot from the first optional one up to `index` must be a non-flag, not just
// `index` itself. Testing them independently - which the C++ did until 2026-08-08 -
// stops a flag being read as a stride but still lets the flag's *value* fall into the
// next slot, so `--press 10:Start` set anchorAddr to 0x10. See §3.50.
fn positional(argv: &[String], index: usize) -> Option<&str> {
    if (5..index).any(|j| argv.get(j).is_none_or(|t| t.starts_with('-'))) {
        return None;
    }
    argv.get(index).map(String::as_str).filter(|t| !t.starts_with('-'))
}

// Beside the dumps and named for whichever backend is driving, so two backends
// probing one ROM never share a profile - see §3.53.
fn home_folder(dump_dir: &str, backend: &str) -> String {
    format!("{dump_dir}/{backend}home")
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let code = run(&argv);
    std::process::exit(code);
}

fn run(argv: &[String]) -> i32 {
    let mut args = match parse_args(argv) {
        Ok(args) => args,
        Err(code) => return code,
    };

    let Some(mut backend) = backends::make(&args.backend_name, &args.core_path) else {
        return 1;
    };
    let backend = backend.as_mut();

    // A libretro core is handed this as its system, save and assets directory
    // and will not create it itself.
    args.options.home_folder = home_folder(&args.dump_dir, backend.name());
    if let Err(error) = std::fs::create_dir_all(&args.options.home_folder) {
        println!("[WARN] could not create {}: {error}", args.options.home_folder);
    }

    backend.set_schedule(args.schedule.clone());

    // Armed before load, which is what starts execution - the reset vector and
    // the whole boot sequence are the point of these two traces.
    if args.cpu_trace_frame != NO_FRAME && !backend.begin_trace(TraceKind::Cpu) {
        println!("[WARN] backend '{}' has no CPU trace hook; --cputrace ignored", backend.name());
        args.cpu_trace_frame = NO_FRAME;
    }
    if args.apu_log_frame != NO_FRAME && !backend.begin_trace(TraceKind::ApuWrites) {
        println!("[WARN] backend '{}' has no APU write hook; --apulog ignored", backend.name());
        args.apu_log_frame = NO_FRAME;
    }

    if !backend.load(&args.rom_path, &args.options) {
        println!("[ERROR] failed to load {}", args.rom_path);
        return 1;
    }

    let prefix = backend.name().to_string();
    let system = backend.system().to_string();
    let identity = backend.identity();

    println!("[INFO] backend {prefix}, system {system}");
    println!(
        "[INFO] identity board={} region={} headerTrust={} prg={} chr={} save={}",
        or_question(&identity.board),
        or_question(&identity.region),
        or_question(&identity.header_trust),
        identity.prg_bytes,
        identity.chr_bytes,
        if identity.save_loaded { 1 } else { 0 }
    );
    for space in &backend.spaces() {
        println!("[INFO]   space {:<10} {} bytes", space.name, space.size());
    }

    let mut signature = open_signature(&args, backend, &prefix, &system, &identity);

    if let Some(code) = run_press_until(&mut args, backend, signature.as_mut()) {
        backend.shutdown();
        return code;
    }

    if args.trace_frame != NO_FRAME {
        backend.begin_trace(TraceKind::Gsu);
    }

    let mut cache = BlobCache::default();
    report_loop(&mut cache, &args, backend, signature.as_mut(), &prefix, &system, &identity);

    if let Some(signature) = signature.as_mut()
        && let Err(error) = signature.flush()
    {
        println!("[WARN] could not flush the signature stream: {error}");
    }

    backend.shutdown();

    if cache.linked > 0 {
        println!("[INFO] {} blob(s) written, {} unchanged and hard-linked", cache.written, cache.linked);
    }

    // After shutdown, which is what closes the recording.
    if let Some(report) = audio::finish(&args.audio_path, args.audio_format)
        && report.encoded
    {
        let percent = 100.0 * report.out_bytes as f64 / report.wav_bytes.max(1) as f64;
        println!(
            "[INFO] audio -> {} ({} bytes, {percent:.1}% of the capture)",
            args.audio_path, report.out_bytes
        );
    }
    0
}

fn or_question(text: &str) -> &str {
    if text.is_empty() { "?" } else { text }
}

fn open_signature(
    args: &Args,
    backend: &mut dyn ProbeBackend,
    prefix: &str,
    system: &str,
    identity: &ProbeIdentity,
) -> Option<SignatureWriter> {
    if !args.want_signature {
        return None;
    }

    let path = std::path::Path::new(&args.dump_dir).join(format!("{prefix}_sig.csv"));
    let mut signature = match SignatureWriter::open(&path) {
        Ok(signature) => signature,
        Err(error) => {
            println!("[WARN] could not open {}: {error}; --sig ignored", path.display());
            return None;
        }
    };

    let spaces = backend.spaces();
    let screen = backend.screen();
    let _ = signature.write_header(prefix, system, &args.rom_path, identity, screen.as_ref(), &spaces);
    let _ = signature.write_row(backend.frame_count(), &spaces, screen.as_ref());
    println!("[INFO] signature stream -> {}", path.display());
    Some(signature)
}

// Anchors the run on game state instead of a frame count, so the same scene
// is reached here and in the harness even though the two emulators do not
// agree on how many frames it takes - see §3.15d and §3.40.
//
// Returns Some(exit code) only when the run must stop here.
fn run_press_until(
    args: &mut Args,
    backend: &mut dyn ProbeBackend,
    mut signature: Option<&mut SignatureWriter>,
) -> Option<i32> {
    let until = args.until.as_ref()?;

    // The anchor is looked up by name on every read rather than held as a
    // reference. The C++ took a pointer into the temporary vector that
    // Spaces() returned by value, so it dangled for the whole loop - it
    // survived only because the bytes it pointed at outlived the vector.
    let anchor_name = backend.anchor_space().to_string();
    if find_space(&backend.spaces(), &anchor_name).is_none() {
        println!("[ERROR] --pressuntil has no {anchor_name} to watch");
        return Some(1);
    }

    let (mut last_press, mut taps, mut frame) = (0u32, 0u32, 0u32);
    let mut reached = false;

    while frame < until.cap {
        frame = backend.frame_count();

        let value = {
            let spaces = backend.spaces();
            find_space(&spaces, &anchor_name).map_or(0, |anchor| anchor_word(anchor, until.addr))
        };
        if value == until.value {
            reached = true;
            break;
        }

        // Held four frames then released; a held button reads as one press to
        // most menus, exactly as the harness's own `tapuntil` does.
        let phase = frame.wrapping_sub(last_press);
        if phase >= until.every {
            backend.set_button(until.button, true);
            last_press = frame;
            taps += 1;
        } else if phase >= 4 {
            backend.set_button(until.button, false);
        }

        advance_to(backend, signature.as_deref_mut(), frame + 1);
    }
    backend.set_button(until.button, false);

    let (addr, value) = (until.addr, until.value);
    if reached {
        println!("[PRESSUNTIL] {anchor_name} ${addr:04X} reached ${value:04X} after {taps} tap(s) at frame {frame}");
    } else {
        println!("[PRESSUNTIL] {anchor_name} ${addr:04X} NOT reached (${value:04X}) after {taps} tap(s), frame {frame}");
    }
    if !reached {
        return Some(2);
    }

    // Report from where the anchor landed; traceUntilFrame is re-read as how
    // many frames of trace after it, and --after is the same span with none.
    args.start_frame = backend.frame_count();
    args.end_frame = args.start_frame
        + if args.trace_frame != NO_FRAME { args.trace_frame } else { args.after_frames };
    if args.trace_frame != NO_FRAME {
        args.trace_frame = args.end_frame;
    }
    None
}

fn report_loop(
    cache: &mut BlobCache,
    args: &Args,
    backend: &mut dyn ProbeBackend,
    mut signature: Option<&mut SignatureWriter>,
    prefix: &str,
    system: &str,
    identity: &ProbeIdentity,
) {
    let mut pending = PendingTraces {
        apu_log_frame: args.apu_log_frame,
        cpu_trace_frame: args.cpu_trace_frame,
        trace_frame: args.trace_frame,
    };
    let mut next_report = args.start_frame;

    loop {
        // A trace ends where it was asked to end. Stopping only at reports made
        // the captured span depend on startFrame rather than on the flag, so
        // `--cputrace 20` recorded 20 frames from startFrame 20 and 60 from
        // startFrame 60 - see §3.52.
        while let Some(due) = pending.earliest_before(backend.frame_count(), next_report) {
            advance_to(backend, signature.as_deref_mut(), due);
            let at = backend.frame_count();
            pending.flush(backend, &args.dump_dir, prefix, at);
        }

        advance_to(backend, signature.as_deref_mut(), next_report);
        let frame = backend.frame_count();

        {
            let spaces = backend.spaces();
            let screen = backend.screen();

            print!("frame {frame:5} | {}", backend.state_line());
            if let Some(anchor) = find_space(&spaces, backend.anchor_space()) {
                let label = anchor.name.to_uppercase();
                println!("{}", dump::hex_line(&label, anchor, args.anchor_addr, 16));
            }

            for space in &spaces {
                if let Err(error) = cache.write(&args.dump_dir, prefix, &space.name, frame, space.data)
                {
                    println!("[WARN] could not write the {} blob: {error}", space.name);
                }
            }
            if let Some(screen) = screen.as_ref() {
                let _ = cache.write(&args.dump_dir, prefix, "screen", frame, screen.data);
            }
            if let Err(error) = dump::write_manifest(
                &args.dump_dir,
                prefix,
                system,
                &args.rom_path,
                frame,
                &spaces,
                screen.as_ref(),
                identity,
            ) {
                println!("[WARN] could not write the manifest: {error}");
            }
        }

        pending.flush(backend, &args.dump_dir, prefix, frame);

        use std::io::Write as _;
        let _ = std::io::stdout().flush();

        next_report = frame + args.stride;
        if frame >= args.end_frame {
            break;
        }
    }
}

struct PendingTraces {
    apu_log_frame: u32,
    cpu_trace_frame: u32,
    trace_frame: u32,
}

impl PendingTraces {
    // The earliest trace end that is still ahead of the machine and falls before
    // `limit`, so the report loop can stop there rather than overrun it.
    //
    // A due frame already behind the machine is deliberately left to the next
    // report. Stopping "at" it would end the trace where it stands and emit an
    // empty file, which is what a `traceUntilFrame` of 0 - the positional's
    // idle value - would otherwise do to every GSU trace.
    fn earliest_before(&self, current: u32, limit: u32) -> Option<u32> {
        [self.apu_log_frame, self.cpu_trace_frame, self.trace_frame]
            .into_iter()
            .filter(|&due| due != NO_FRAME && due > current && due < limit)
            .min()
    }

    fn flush(&mut self, backend: &mut dyn ProbeBackend, dir: &str, prefix: &str, frame: u32) {
        let kinds: [(&mut u32, TraceKind, &str, &[u8; 8], usize); 3] = [
            (&mut self.apu_log_frame, TraceKind::ApuWrites, "apulog", b"ESAW\x01\x00\x00\x00", 12),
            (&mut self.cpu_trace_frame, TraceKind::Cpu, "cputrace", b"ESCT\x02\x00\x00\x00", 24),
            (&mut self.trace_frame, TraceKind::Gsu, "gsutrace", b"ESGT\x01\x00\x00\x00", 48),
        ];

        for (due, kind, name, magic, record_size) in kinds {
            if *due == NO_FRAME || frame < *due {
                continue;
            }
            // Cleared whether or not the backend produced anything. end_trace
            // returning None is a statement about the backend's capabilities,
            // not a transient failure, so retrying can only spin - and once the
            // loop above stops *at* a due frame, a due that never clears is an
            // infinite loop rather than a harmless retry.
            let trace = backend.end_trace(kind);
            *due = NO_FRAME;
            if let Some(trace) = trace
                && let Err(error) =
                    dump::write_trace(dir, prefix, name, frame, magic, &trace, record_size)
            {
                println!("[WARN] could not write the {name} trace: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The header's claim that these layers name no emulator, made checkable -
    // see EmuSen_Debugging_Tools_Reference_v5.md §3.53.
    const AGNOSTIC: [(&str, &str); 4] = [
        ("main.rs", include_str!("main.rs")),
        ("backend.rs", include_str!("backend.rs")),
        ("dump.rs", include_str!("dump.rs")),
        ("audio.rs", include_str!("audio.rs")),
    ];

    // Every emulator this probe has ever had a backend for or been compared
    // against. A name is added here when a backend is added, never removed.
    const EMULATORS: [&str; 9] = [
        "mesen", "libretro", "nestopia", "gambatte", "bsnes", "snes9x", "retroarch", "bizhawk",
        "ares",
    ];

    // Comments may name a backend to explain why it exists; code may not. The
    // scan stops at the test module, whose fixtures use real names on purpose.
    fn code_lines(source: &str) -> Vec<(usize, &str)> {
        source
            .lines()
            .take_while(|line| !line.starts_with("#[cfg(test)]"))
            .enumerate()
            .map(|(n, line)| (n + 1, line.split("//").next().unwrap_or("")))
            .filter(|(_, code)| !code.trim().is_empty())
            .collect()
    }

    #[test]
    fn no_emulator_is_named_in_the_agnostic_layers() {
        let mut found = Vec::new();
        for (file, source) in AGNOSTIC {
            for (number, code) in code_lines(source) {
                let lowered = code.to_lowercase();
                if EMULATORS.iter().any(|e| lowered.contains(e)) {
                    found.push(format!("  {file}:{number}: {}", code.trim()));
                }
            }
        }
        assert!(
            found.is_empty(),
            "the policy layer names an emulator in {} place(s):\n{}",
            found.len(),
            found.join("\n")
        );
    }

    fn pending(apu: u32, cpu: u32, gsu: u32) -> PendingTraces {
        PendingTraces { apu_log_frame: apu, cpu_trace_frame: cpu, trace_frame: gsu }
    }

    // A trace must end on the frame it was asked for, even when the first report
    // is much later - otherwise the captured span is a function of startFrame.
    #[test]
    fn a_trace_due_before_the_next_report_stops_the_loop_there() {
        let p = pending(NO_FRAME, 20, NO_FRAME);
        assert_eq!(p.earliest_before(1, 60), Some(20));
        // Two due at once: the earlier one first, and the loop runs again.
        let p = pending(30, 20, NO_FRAME);
        assert_eq!(p.earliest_before(1, 60), Some(20));
        assert_eq!(p.earliest_before(20, 60), Some(30));
    }

    // The regression this test exists for: `traceUntilFrame` is a positional
    // whose idle value is 0, and treating a frame already behind the machine as
    // a stopping point ended every GSU trace at once and wrote an empty file.
    #[test]
    fn a_trace_due_in_the_past_is_left_to_the_next_report() {
        let p = pending(NO_FRAME, NO_FRAME, 0);
        assert_eq!(p.earliest_before(1, 80), None);
        let p = pending(NO_FRAME, NO_FRAME, 40);
        assert_eq!(p.earliest_before(40, 80), None, "the due frame itself is not ahead");
        assert_eq!(p.earliest_before(39, 80), Some(40));
    }

    #[test]
    fn a_trace_due_at_or_after_the_next_report_does_not_stop_the_loop() {
        let p = pending(NO_FRAME, 60, NO_FRAME);
        assert_eq!(p.earliest_before(1, 60), None);
        assert_eq!(p.earliest_before(1, 61), Some(60));
        assert_eq!(pending(NO_FRAME, NO_FRAME, NO_FRAME).earliest_before(1, 9999), None);
    }

    fn args(text: &[&str]) -> Vec<String> {
        text.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn atoi_stops_at_the_first_non_digit() {
        assert_eq!(atoi("120"), 120);
        assert_eq!(atoi("12abc"), 12);
        assert_eq!(atoi("abc"), 0);
        assert_eq!(atoi(""), 0);
        assert_eq!(atoi("  7 "), 7);
    }

    #[test]
    fn strtoul_hex_reads_hex_with_or_without_prefix() {
        assert_eq!(strtoul_hex("0A00"), 0x0A00);
        assert_eq!(strtoul_hex("0x1e1a"), 0x1E1A);
        assert_eq!(strtoul_hex("ffff"), 0xFFFF);
        assert_eq!(strtoul_hex("zz"), 0);
    }

    // The quirk that lets --press follow endFrame directly: a leading '-' keeps
    // that argument from being read as a stride.
    #[test]
    fn a_flag_is_not_read_as_a_positional() {
        let argv = args(&["probe", "r.nes", "/out", "0", "120", "--press", "10:Start"]);
        let parsed = parse_args(&argv).ok().unwrap();
        assert_eq!(parsed.stride, 1);
        assert_eq!(parsed.trace_frame, NO_FRAME);
        assert_eq!(parsed.schedule.entries.len(), 1);
        assert!(parsed.schedule.held_at(13, ProbeButton::Start));
        assert!(!parsed.schedule.held_at(14, ProbeButton::Start));
    }

    // The first flag now genuinely ends the optional positionals. Until 2026-08-08
    // each slot was tested on its own, so `--press 10:Start` set anchorAddr to 0x10
    // and `--backend libretro` set it to 0; both now leave the default standing.
    // See §3.50's note on the positional quirk.
    #[test]
    fn a_flag_ends_the_run_of_positionals() {
        for argv in [
            args(&["probe", "r.nes", "/out", "0", "120", "--press", "10:Start"]),
            args(&["probe", "r.nes", "/out", "0", "120", "--backend", "libretro"]),
            args(&["probe", "r.nes", "/out", "0", "120", "--sig"]),
            args(&["probe", "r.nes", "/out", "0", "120", "--pressuntil", "A:1E1A:F001"]),
        ] {
            let parsed = parse_args(&argv).ok().unwrap();
            assert_eq!(parsed.stride, 1, "{argv:?}");
            assert_eq!(parsed.anchor_addr, 0x0A00, "{argv:?}");
            assert_eq!(parsed.trace_frame, NO_FRAME, "{argv:?}");
        }
    }

    // A positional-looking argument after a flag does not resume the run: the
    // trailing 1E1A here is a stray, not an anchor address.
    #[test]
    fn positionals_do_not_resume_after_a_flag() {
        let argv = args(&["probe", "r.nes", "/out", "0", "120", "--after", "8", "1E1A"]);
        let parsed = parse_args(&argv).ok().unwrap();
        assert_eq!(parsed.after_frames, 8);
        assert_eq!(parsed.anchor_addr, 0x0A00);
        assert_eq!(parsed.trace_frame, NO_FRAME);
    }

    // The stride slot still works when it is genuinely a positional, and a flag
    // after it still does not leak into the anchor slot.
    #[test]
    fn a_stride_then_a_flag_keeps_the_anchor_default() {
        let argv = args(&["probe", "r.nes", "/out", "0", "120", "4", "--sig"]);
        let parsed = parse_args(&argv).ok().unwrap();
        assert_eq!(parsed.stride, 4);
        assert_eq!(parsed.anchor_addr, 0x0A00);
        assert!(parsed.want_signature);
    }

    #[test]
    fn positionals_are_read_when_present() {
        let argv = args(&["probe", "r.nes", "/out", "60", "180", "4", "1E1A", "90"]);
        let parsed = parse_args(&argv).ok().unwrap();
        assert_eq!((parsed.start_frame, parsed.end_frame), (60, 180));
        assert_eq!(parsed.stride, 4);
        assert_eq!(parsed.anchor_addr, 0x1E1A);
        assert_eq!(parsed.trace_frame, 90);
    }

    #[test]
    fn press_until_defaults_match_the_cpp() {
        let argv = args(&["probe", "r.sfc", "/out", "0", "0", "--pressuntil", "A:1E1A:F001"]);
        let until = parse_args(&argv).ok().unwrap().until.unwrap();
        assert_eq!(until.button, ProbeButton::A);
        assert_eq!((until.addr, until.value), (0x1E1A, 0xF001));
        assert_eq!((until.cap, until.every), (6000, 40));
    }

    #[test]
    fn press_until_takes_cap_and_every() {
        let argv = args(&["probe", "r.sfc", "/out", "0", "0", "--pressuntil", "A:1E1A:F001:12000:40"]);
        let until = parse_args(&argv).ok().unwrap().until.unwrap();
        assert_eq!((until.cap, until.every), (12000, 40));
    }

    #[test]
    fn too_few_arguments_is_exit_one() {
        assert_eq!(parse_args(&args(&["probe", "r.nes", "/out"])).err(), Some(1));
    }

    #[test]
    fn an_unknown_button_is_exit_one() {
        let argv = args(&["probe", "r.nes", "/out", "0", "1", "--press", "10:Turbo"]);
        assert_eq!(parse_args(&argv).err(), Some(1));
    }

    // Until 2026-08-09 the policy layer hardcoded "mesenhome" here, so a
    // libretro core was handed a save directory named after a different
    // emulator entirely. The mesen path is unchanged, which is why an existing
    // dump set keeps working - see §3.53.
    #[test]
    fn the_home_folder_is_named_for_the_backend_driving_it() {
        assert_eq!(home_folder("/out/dir", "mesen"), "/out/dir/mesenhome");
        assert_eq!(home_folder("/out/dir", "libretro"), "/out/dir/libretrohome");
    }

    #[test]
    fn parsing_alone_does_not_choose_a_home_folder() {
        let argv = args(&["probe", "r.nes", "/out/dir", "0", "1"]);
        let parsed = parse_args(&argv).ok().unwrap();
        assert_eq!(parsed.options.home_folder, "");
        assert_eq!(parsed.options.ram_state, "zeros");
    }

    // A power-of-two space mirrors rather than crashing, which is what makes an
    // out-of-range --pressuntil address survivable.
    #[test]
    fn anchor_word_mirrors_past_the_end() {
        let data: Vec<u8> = (0..=255u8).collect();
        let space = MemorySpace { name: "ram".to_string(), data: &data };
        assert_eq!(anchor_word(&space, 0), 0x0100);
        assert_eq!(anchor_word(&space, 0x10), 0x1110);
        assert_eq!(anchor_word(&space, 256), 0x0100);
        assert_eq!(anchor_word(&space, 255), 0x00FF);
    }

    #[test]
    fn anchor_word_on_an_empty_space_is_zero() {
        let space = MemorySpace { name: "ram".to_string(), data: &[] };
        assert_eq!(anchor_word(&space, 0), 0);
    }
}
