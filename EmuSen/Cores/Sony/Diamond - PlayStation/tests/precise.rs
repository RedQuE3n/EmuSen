//! Stage E4's gate (Diamond_Enhancements.md §23): precise geometry, the precision layer paired with the renderer,
//! attached from power-on. The machine's goldens with it on; the precise record at zero fraction against the software
//! rasteriser on every run E3's gate makes; a textured grid turned slowly through the GTE, its vertices' jitter and its
//! texture's warp with the setting off and on; a 2D program drawn as the software rasteriser draws it; the state both
//! ways; and on the bench discs the fraction of vertices drawn precisely. A machine without a Vulkan device stands each
//! test that needs one down with a line.

use std::any::Any;

use blackmoon_gte::Observer;
use diamond::gpu::Gpu;
use diamond::gpu::decode;
use diamond::layer::{Command, Layer, Part, Shown};
use diamond::machine::{Config, Gpu as Chip, Machine, RAM_ID};
use diamond::media::{Market, Media};
use diamond_renderer::Renderer;
use emusen_native::StateWriter;
use emusen_native::ffi::StateMachine;
use precision::{Counts, Draw, Form, Fraction, Precise, Precision};

#[path = "support/goldens.rs"]
mod goldens;
use goldens::*;

#[path = "support/records.rs"]
mod records;

fn renderer() -> Option<Renderer> {
    match Renderer::new(None) {
        Ok(r) => Some(r.keeping(true)),
        Err(e) => {
            eprintln!("no Vulkan device ({e}): not run");
            None
        }
    }
}

/// The drawer of the gate: the renderer, when there is one, and the polygons it was given with their precise vertices,
/// the drawing offset they were decoded under.
struct Tap {
    renderer: Option<Renderer>,
    log: bool,
    polygons: Vec<(Vec<u32>, u32, Option<[Fraction; 4]>)>,
}

impl Tap {
    fn new(renderer: Option<Renderer>, log: bool) -> Tap {
        Tap { renderer, log, polygons: Vec::new() }
    }
}

impl Observer for Tap {}

impl Layer for Tap {
    fn shown(&mut self, s: Shown) {
        if let Some(r) = &mut self.renderer {
            r.shown(s);
        }
    }
    fn seed(&mut self, gpu: &Gpu) {
        if let Some(r) = &mut self.renderer {
            r.seed(gpu);
        }
    }
    fn tags(&self) -> Vec<[u8; 4]> {
        self.renderer.as_ref().map_or(Vec::new(), |r| r.tags())
    }
    fn save(&self, tag: [u8; 4], w: &mut StateWriter) {
        if let Some(r) = &self.renderer {
            r.save(tag, w);
        }
    }
    fn load(&mut self, tag: [u8; 4], block: &[u8]) {
        if let Some(r) = &mut self.renderer {
            r.load(tag, block);
        }
    }
    fn clear(&mut self, tag: [u8; 4]) {
        if let Some(r) = &mut self.renderer {
            r.clear(tag);
        }
    }
    fn settle(&mut self) {
        if let Some(r) = &mut self.renderer {
            r.settle();
        }
    }
}

impl Draw for Tap {
    fn executed_with(&mut self, c: &Command, precise: Option<&[Fraction; 4]>) {
        if self.log && c.part == Part::Whole && c.words[0] >> 29 == 1 {
            self.polygons.push((c.words.to_vec(), c.drawing.env[4], precise.copied()));
        }
        if let Some(r) = &mut self.renderer {
            r.executed_with(c, precise);
        }
    }
}

fn tap(m: &mut Machine) -> &mut Precise<Tap> {
    let layer: &mut dyn Any = m.layer().expect("attached");
    layer.downcast_mut::<Precise<Tap>>().expect("the pair")
}

fn drawn(m: &mut Machine) -> &mut Renderer {
    tap(m).drawer.renderer.as_mut().expect("a renderer")
}

/// The renderer's picture against the machine's displayed one, and its VRAM against the GPU's.
#[derive(Clone, Debug, Default)]
struct Compared {
    frames: u64,
    frames_differing: u64,
    pixels: u64,
    first: Option<i64>,
    vram: u64,
}

impl Compared {
    fn frame(&mut self, m: &mut Machine) {
        let frames = m.frames;
        let made: Vec<u8> = m.board.gpu.display.frame().to_vec();
        let (w, h) = (m.board.gpu.display.width, m.board.gpu.display.height);
        let (pic, rw, rh) = drawn(m).picture();
        self.frames += 1;
        let differing = if (rw, rh) != (w, h) { (w * h) as u64 } else { made.chunks(4).zip(pic.chunks(4)).filter(|(a, b)| a != b).count() as u64 };
        if differing > 0 {
            self.frames_differing += 1;
            self.pixels += differing;
            self.first.get_or_insert(frames);
        }
    }

    fn vram(&mut self, m: &mut Machine) {
        let mut v = vec![0u16; 1024 * 512];
        drawn(m).vram(&mut v);
        self.vram += v.iter().enumerate().filter(|&(i, &d)| m.board.gpu.vram.get(i as u32 % 1024, i as u32 / 1024) != d).count() as u64;
    }
}

fn goldens_equal(file: &str, made: &[String]) -> Vec<String> {
    let kept = kept_goldens(file);
    let mut differ: Vec<String> = made.iter().zip(&kept).filter(|(a, b)| a != b).map(|(a, b)| format!("  made {a}\n  kept {b}")).collect();
    if made.len() != kept.len() {
        differ.push(format!("  {} lines made, {} kept", made.len(), kept.len()));
    }
    differ
}

fn counts_line(c: &Counts, r: Option<&diamond_renderer::Counts>) -> String {
    let pct = |a: u64, b: u64| if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 };
    let mut s = format!(
        "projections {} (cut {}, {:.2}%; far {}, {:.2}%); polygons {} precise {} ({:.1}%) mixed {}; vertices {} precise {} ({:.1}%); dead {} changed {} unmatched {}",
        c.projections, c.refused, pct(c.refused, c.projections), c.far, pct(c.far, c.projections), c.polygons, c.precise_polygons, pct(c.precise_polygons, c.polygons), c.mixed_polygons, c.vertices, c.precise_vertices, pct(c.precise_vertices, c.vertices), c.dead, c.changed, c.unmatched
    );
    if let Some(r) = r {
        s += &format!("; renderer: precise triangles {} perspective {} integer-fallback polygons {}", r.precise, r.perspective, r.integer);
    }
    s
}

/// The texture cache's halfwords, as E3's gate pins them (Diamond_Enhancements.md §17.2).
const TEXTURE_CACHE_HALFWORDS: [(&str, u64); 2] = [("texture-cache.s", 128), ("triangle-rows.s", 3680)];

/// `m` run `frames` frames with the pair attached in `form`, the renderer's picture compared at every frame and its
/// VRAM at the end; the run's golden digests with the pair attached.
fn run_paired(mut m: Machine, r: Renderer, form: Form, frames: usize) -> (Compared, Counts, diamond_renderer::Counts, String) {
    m.attach(Box::new(Precise::new(Precision::new(form), Tap::new(Some(r), false))));
    let (mut c, mut fold) = (Compared::default(), Fold::new());
    for _ in 0..frames {
        m.advance();
        fold.frame(&mut m);
        c.frame(&mut m);
    }
    c.vram(&mut m);
    let counts = tap(&mut m).precision.counts();
    (c, counts, drawn(&mut m).counts(), end_digests(&m, &fold))
}

/// §23's first two items on the project's programs, each as its golden runs it: with the setting on (the exact form)
/// every golden line holds; with the zero form every picture and VRAM is the software rasteriser's, as E3's gate holds.
#[test]
fn the_programs_keep_their_goldens_and_are_drawn_exactly_at_zero_fraction() {
    if renderer().is_none() {
        return;
    }
    let (mut failed, mut on, mut zero) = (Vec::new(), Vec::new(), Vec::new());
    for (name, source) in own_programs() {
        let (_, counts, _, digests) = run_paired(golden_program(&name, &source), renderer().unwrap(), Form::Exact, GOLDEN_PROGRAM_FRAMES);
        on.push(format!("{name}\t{GOLDEN_PROGRAM_FRAMES}\t{digests}"));
        let (c, zc, rc, digests) = run_paired(golden_program(&name, &source), renderer().unwrap(), Form::Zero, GOLDEN_PROGRAM_FRAMES);
        zero.push(format!("{name}\t{GOLDEN_PROGRAM_FRAMES}\t{digests}"));
        let cache = TEXTURE_CACHE_HALFWORDS.iter().find(|p| p.0 == name).map_or(0, |p| p.1);
        if c.pixels != 0 || c.vram != cache {
            failed.push(format!("{name}: {c:?}"));
        }
        if counts.projections > 0 || zc.precise_polygons > 0 {
            eprintln!("{name}: on: {}\n  zero: {}", counts_line(&counts, None), counts_line(&zc, Some(&rc)));
        }
    }
    let (a, b) = (goldens_equal("programs.txt", &on), goldens_equal("programs.txt", &zero));
    assert!(failed.is_empty() && a.is_empty() && b.is_empty(), "at zero fraction:\n{}\ngoldens on:\n{}\nzero:\n{}", failed.join("\n"), a.join("\n"), b.join("\n"));
}

/// The same on ps1-tests' release, for the frames its goldens run.
#[test]
fn ps1_tests_keep_their_goldens_and_are_drawn_exactly_at_zero_fraction() {
    let Some(corpus) = records::corpus() else {
        eprintln!("not run: {} names no ps1-tests build 158", records::CORPUS_VARIABLE);
        return;
    };
    if renderer().is_none() {
        return;
    }
    let (mut failed, mut on, mut zero) = (Vec::new(), Vec::new(), Vec::new());
    for path in ps1_tests_programs(&corpus) {
        let (frames, market) = ps1_tests_run(&path);
        let (_, counts, _, digests) = run_paired(ps1_tests_machine(&corpus, &path, market), renderer().unwrap(), Form::Exact, frames);
        on.push(format!("{path}\t{frames}\t{digests}"));
        let (c, zc, rc, digests) = run_paired(ps1_tests_machine(&corpus, &path, market), renderer().unwrap(), Form::Zero, frames);
        zero.push(format!("{path}\t{frames}\t{digests}"));
        if c.pixels != 0 || c.vram != 0 {
            failed.push(format!("{path}: {c:?}"));
        }
        if counts.projections > 0 {
            eprintln!("{path}: on: {}\n  zero: {}", counts_line(&counts, None), counts_line(&zc, Some(&rc)));
        }
    }
    let (a, b) = (goldens_equal("ps1-tests.txt", &on), goldens_equal("ps1-tests.txt", &zero));
    assert!(failed.is_empty() && a.is_empty() && b.is_empty(), "at zero fraction:\n{}\ngoldens on:\n{}\nzero:\n{}", failed.join("\n"), a.join("\n"), b.join("\n"));
}

/// The bench discs by their press scripts to frame 3,000, the pair attached from power-on in `form`: the goldens'
/// digests at every anchor, the renderer's picture against the machine's at every frame and its VRAM at every anchor,
/// and the precision layer's counts. With `EMUSEN_DIAMOND_PICTURES` a folder, the exact form's pictures at each anchor
/// are written there for looking at, the renderer's and the machine's side by side as raw RGBA.
fn discs(form: Form) -> Option<(Vec<String>, Vec<String>)> {
    let Some(folder) = std::env::var_os("EMUSEN_DIAMOND_DISCS") else {
        eprintln!("not run: EMUSEN_DIAMOND_DISCS names no folder of the bench discs");
        return None;
    };
    renderer()?;
    let pictures = std::env::var_os("EMUSEN_DIAMOND_PICTURES").filter(|_| form == Form::Exact).map(std::path::PathBuf::from);
    let folder = std::path::PathBuf::from(folder);
    let (mut lines, mut goldens) = (Vec::new(), Vec::new());
    for (file, script) in GOLDEN_DISCS {
        let (anchor, presses, sticks) = press_script(script);
        goldens.push(format!("{file}\t{script}\tanchor {anchor}"));
        let mut m = disc_maker(folder.join(file))();
        m.insert_card(0, None);
        m.attach(Box::new(Precise::new(Precision::new(form), Tap::new(renderer(), false))));
        let (mut c, mut fold) = (Compared::default(), Fold::new());
        let mut by_anchor = Vec::new();
        while m.frames < GOLDEN_DISC_FRAMES {
            press(&mut m, &presses, &sticks);
            m.advance();
            fold.frame(&mut m);
            c.frame(&mut m);
            if m.frames % GOLDEN_DISC_ANCHOR == 0 {
                c.vram(&mut m);
                goldens.push(format!("\t{}\tpicture {:016X}\tsound {:016X}\tstate {:016X}", m.frames, fold.picture, fold.sound, state_digest(&m)));
                let k = tap(&mut m).precision.counts();
                by_anchor.push(format!("{}:{}/{}", m.frames, k.precise_vertices, k.vertices));
                if let Some(dir) = &pictures {
                    let stem = format!("{}-{:05}", script.trim_end_matches(".txt"), m.frames);
                    let (w, h) = (m.board.gpu.display.width, m.board.gpu.display.height);
                    std::fs::write(dir.join(format!("{stem}.machine.{w}x{h}.rgba")), m.board.gpu.display.frame()).unwrap();
                    let (pic, rw, rh) = drawn(&mut m).picture();
                    std::fs::write(dir.join(format!("{stem}.precise.{rw}x{rh}.rgba")), pic).unwrap();
                }
            }
        }
        let k = tap(&mut m).precision.counts();
        let r = drawn(&mut m).counts();
        lines.push(format!("{file}: {}; pictures differing in {} of {} frames, {} pixels, first {:?}; VRAM {}; by anchor {}", counts_line(&k, Some(&r)), c.frames_differing, c.frames, c.pixels, c.first, c.vram, by_anchor.join(" ")));
        eprintln!("{}", lines.last().unwrap());
        if form == Form::Zero && (c.pixels != 0 || c.vram != 0) {
            lines.push(format!("FAILED {file}: at zero fraction the renderer differs"));
        }
    }
    Some((lines, goldens))
}

/// §23 on the bench discs with the setting on: every golden anchor's digests, and the counts per game (P10).
#[test]
fn the_bench_discs_keep_their_goldens_with_the_setting_on() {
    let Some((lines, goldens)) = discs(Form::Exact) else { return };
    let differ = goldens_equal("discs.txt", &goldens);
    assert!(differ.is_empty(), "goldens with the setting on:\n{}\n{}", differ.join("\n"), lines.join("\n"));
}

/// §23 on the bench discs at zero fraction: every picture and every anchor's VRAM the software rasteriser's.
#[test]
fn the_bench_discs_are_drawn_exactly_at_zero_fraction() {
    let Some((lines, goldens)) = discs(Form::Zero) else { return };
    let differ = goldens_equal("discs.txt", &goldens);
    assert!(differ.is_empty() && !lines.iter().any(|l| l.starts_with("FAILED")), "{}\n{}", differ.join("\n"), lines.join("\n"));
}

// --- The synthetic programs ---

fn li(r: &str, v: u32) -> String {
    format!("lui {r}, 0x{:X}\nori {r}, {r}, 0x{:X}\n", v >> 16, v & 0xFFFF)
}

fn gp0(v: u32) -> String {
    li("t0", v) + "sw t0, 0x1810(a0)\n"
}

fn gp1(v: u32) -> String {
    li("t0", v) + "sw t0, 0x1814(a0)\n"
}

fn machine(source: &str) -> Machine {
    let words = emusen_mips::assemble(source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = diamond::exe::build(&words);
    Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Chip::Pin208 }, [Some("psx.dualshock"), None], &image, None)
}

/// The texture page at (512, 0), 15-bit, every texel naming its coordinate: bit 15, u in bits 0-7, v in bits 8-14.
fn coordinate_texture(m: &mut Machine) {
    for v in 0..128 {
        for u in 0..256 {
            m.board.gpu.vram.set(512 + u, v, 0x8000 | u as u16 | (v as u16) << 8);
        }
    }
}

const H: u32 = 280;
const GRID: usize = 5;
const STEP: i32 = 200;
const TR: [i32; 3] = [0, 100, 1300];
const TILT: f64 = -1.0;
const TURN: f64 = 0.004;
const MESH_FRAMES: usize = 120;

/// The grid's rotation at frame `k`: tilted about x, turned about y by `k` steps.
fn rotation(k: usize) -> [[f64; 3]; 3] {
    let (t, a) = (TURN * k as f64, TILT);
    let ry = [[t.cos(), 0.0, t.sin()], [0.0, 1.0, 0.0], [-t.sin(), 0.0, t.cos()]];
    let rx = [[1.0, 0.0, 0.0], [0.0, a.cos(), -a.sin()], [0.0, a.sin(), a.cos()]];
    std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|n| rx[i][n] * ry[n][j]).sum()))
}

fn grid_vertex(i: usize) -> [i32; 3] {
    let half = (GRID as i32 - 1) * STEP / 2;
    [(i % GRID) as i32 * STEP - half, 0, (i / GRID) as i32 * STEP - half]
}

/// The quads' vertices in the grid, and their texture coordinates in the PlayStation's quad order.
fn quads() -> Vec<[usize; 4]> {
    (0..GRID - 1).flat_map(|j| (0..GRID - 1).map(move |i| [j * GRID + i, j * GRID + i + 1, (j + 1) * GRID + i, (j + 1) * GRID + i + 1])).collect()
}

const UV: [(u32, u32); 4] = [(0, 0), (63, 0), (0, 63), (63, 63)];

/// The ideal screen position and depth of grid vertex `i` at frame `k`, in the buffer the frame is drawn into.
fn ideal(i: usize, k: usize) -> (f64, f64, f64) {
    let (r, v) = (rotation(k), grid_vertex(i));
    let p: [f64; 3] = std::array::from_fn(|n| (0..3).map(|m| r[n][m] * v[m] as f64).sum::<f64>() + TR[n] as f64);
    (160.0 + H as f64 * p[0] / p[2], 120.0 + H as f64 * p[1] / p[2] + 256.0 * (k & 1) as f64, p[2])
}

/// The grid program (§23): after each vertical blank it shows the buffer it drew last, sets the drawing area and
/// offset to the other, clears it, loads the frame's rotation from a table, projects each quad's four vertices by RTPT
/// and RTPS into its packet in a DMA list, sends the list, and counts the frame.
fn mesh_program() -> String {
    let mut s = String::new();
    s += "lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\n";
    s += &gp1(0);
    s += &gp1(0x0800_0001);
    s += &gp1(0x0300_0000);
    s += &gp1(0x0400_0002);
    s += &li("t0", 0x0765_4B21);
    s += "sw t0, 0x10F0(a0)\n";
    s += &li("t0", H);
    s += "ctc2 t0, $26\nlui t0, 160\nctc2 t0, $24\nlui t0, 120\nctc2 t0, $25\n";
    s += &gp0(0xE100_0508);
    s += &gp0(0xE600_0000);
    s += "lui s3, 0x8010\nlui s4, 0x8012\nori s1, zero, 0\n";
    s += "frame:\nwait: lw t0, 0x1070(a0)\nnop\nandi t0, t0, 1\nbeq t0, zero, wait\nnop\n";
    s += &li("t0", 0xFFFF_FFFE);
    s += "sw t0, 0x1070(a0)\nandi t1, s1, 1\nxori t2, t1, 1\nsll t3, t1, 18\n";
    s += "sll t4, t2, 18\nlui t0, 0x0500\nor t0, t0, t4\nsw t0, 0x1814(a0)\n";
    s += "lui t0, 0xE300\nor t0, t0, t3\nsw t0, 0x1810(a0)\n";
    s += &li("t0", 0xE400_0000 | 319 | 239 << 10);
    s += "or t0, t0, t3\nsw t0, 0x1810(a0)\nsll t4, t1, 19\nlui t0, 0xE500\nor t0, t0, t4\nsw t0, 0x1810(a0)\n";
    s += &gp0(0x0230_3030);
    s += "sll t4, t1, 24\nsw t4, 0x1810(a0)\n";
    s += &gp0(0x00F0_0140);
    s += "sll t4, s1, 5\nlui s2, 0x8011\naddu s2, s2, t4\n";
    for r in 0..8 {
        s += &format!("lw t0, {}(s2)\nnop\nctc2 t0, ${r}\n", 4 * r);
    }
    for (q, v) in quads().iter().enumerate() {
        let p = 40 * q;
        s += &format!("lwc2 $0, {}(s3)\nlwc2 $1, {}(s3)\nlwc2 $2, {}(s3)\nlwc2 $3, {}(s3)\nlwc2 $4, {}(s3)\nlwc2 $5, {}(s3)\nnop\n.word 0x4A280030\n", 8 * v[0], 8 * v[0] + 4, 8 * v[1], 8 * v[1] + 4, 8 * v[2], 8 * v[2] + 4);
        s += &format!("swc2 $12, {}(s4)\nswc2 $13, {}(s4)\nswc2 $14, {}(s4)\n", p + 8, p + 16, p + 24);
        s += &format!("lwc2 $0, {}(s3)\nlwc2 $1, {}(s3)\nnop\n.word 0x4A180001\nswc2 $14, {}(s4)\n", 8 * v[3], 8 * v[3] + 4, p + 32);
    }
    s += &li("t0", 0x0012_0000);
    s += "sw t0, 0x10A0(a0)\nsw zero, 0x10A4(a0)\n";
    s += &li("t0", 0x0100_0401);
    s += "sw t0, 0x10A8(a0)\ndma: lw t0, 0x10A8(a0)\nnop\nlui t3, 0x0100\nand t0, t0, t3\nbne t0, zero, dma\nnop\n";
    s += "addiu s1, s1, 1\nlui t0, 0x8013\nsw s1, 0(t0)\nj frame\nnop\n";
    s
}

fn poke(m: &mut Machine, address: u32, words: &[u32]) {
    let ram = m.space_mut(RAM_ID).unwrap();
    for (i, w) in words.iter().enumerate() {
        let a = (address as usize & 0x1F_FFFF) + 4 * i;
        ram[a..a + 4].copy_from_slice(&w.to_le_bytes());
    }
}

/// The grid program's machine: its vertices, every frame's rotation in fixed point, its packets' fixed words and the
/// coordinate texture put in place.
fn mesh_machine() -> Machine {
    let mut m = machine(&mesh_program());
    let vertices: Vec<u32> = (0..GRID * GRID).flat_map(|i| {
        let v = grid_vertex(i);
        [v[0] as u16 as u32 | (v[1] as u16 as u32) << 16, v[2] as u16 as u32]
    }).collect();
    poke(&mut m, 0x8010_0000, &vertices);
    let fixed = |x: f64| (x * 4096.0).round() as i32 as u16 as u32;
    let rotations: Vec<u32> = (0..MESH_FRAMES + 8).flat_map(|k| {
        let r = rotation(k);
        [fixed(r[0][0]) | fixed(r[0][1]) << 16, fixed(r[0][2]) | fixed(r[1][0]) << 16, fixed(r[1][1]) | fixed(r[1][2]) << 16, fixed(r[2][0]) | fixed(r[2][1]) << 16, fixed(r[2][2]), TR[0] as u32, TR[1] as u32, TR[2] as u32]
    }).collect();
    poke(&mut m, 0x8011_0000, &rotations);
    let n = quads().len();
    for q in 0..n {
        let next = if q + 1 == n { 0xFF_FFFF } else { 0x12_0000 + 40 * (q as u32 + 1) };
        let uv = |c: usize, attribute: u32| UV[c].0 | UV[c].1 << 8 | attribute << 16;
        poke(&mut m, 0x8012_0000 + 40 * q as u32, &[9 << 24 | next, 0x2D80_8080, 0, uv(0, 0), 0, uv(1, 0x108), 0, uv(2, 0), 0, uv(3, 0)]);
    }
    coordinate_texture(&mut m);
    m
}

fn frames_drawn(m: &mut Machine) -> usize {
    let ram = m.space(RAM_ID).unwrap();
    u32::from_le_bytes(ram[0x13_0000..0x13_0004].try_into().unwrap()) as usize
}

/// The grid's vertices as drawn at each frame, by grid vertex: the integer the GPU decoded and the precise offset, by
/// the polygons the drawer was given in their order, sixteen a frame.
fn vertices_drawn(polygons: &[(Vec<u32>, u32, Option<[Fraction; 4]>)]) -> Vec<Vec<((f64, f64), Option<(f64, f64)>)>> {
    let q = quads();
    polygons.chunks_exact(q.len()).map(|frame| {
        let mut out = vec![((0.0, 0.0), None); GRID * GRID];
        for (quad, (words, offset, precise)) in q.iter().zip(frame) {
            let p = decode::polygon(words, *offset);
            for c in 0..4 {
                out[quad[c]] = ((p.v[c].x as f64, p.v[c].y as f64), precise.map(|f| (p.v[c].x as f64 + f[c].dx as f64 / 256.0, p.v[c].y as f64 + f[c].dy as f64 / 256.0)));
            }
        }
        out
    }).collect()
}

/// The mean screen-position error against the ideal projection, and the mean change of that error from one frame to the
/// next: the jitter a slowly turning grid shows.
fn jitter(frames: &[Vec<(f64, f64)>]) -> (f64, f64, f64) {
    let error = |k: usize, i: usize| {
        let (x, y, _) = ideal(i, k);
        (frames[k][i].0 - x, frames[k][i].1 - y)
    };
    let (mut e, mut d, mut worst, mut n, mut nd) = (0.0, 0.0, 0.0f64, 0.0, 0.0);
    for k in 0..frames.len() {
        for i in 0..GRID * GRID {
            let a = error(k, i);
            let len = a.0.hypot(a.1);
            e += len;
            worst = worst.max(len);
            n += 1.0;
            if k > 0 {
                let b = error(k - 1, i);
                d += (a.0 - b.0).hypot(a.1 - b.1);
                nd += 1.0;
            }
        }
    }
    (e / n, d / nd, worst)
}

/// The texel error of frame `k`'s picture against the perspective-correct reference: for every pixel at least 1.5 pixels
/// inside an ideal triangle of a quad, the texel drawn (decoded from the coordinate texture) against the texel the
/// geometry puts there; the mean error in texels and the share of pixels more than a texel off.
fn warp(vram: impl Fn(u32, u32) -> u16, k: usize, sums: &mut (f64, f64, f64)) {
    for quad in quads() {
        for tri in [[0, 1, 2], [1, 2, 3]] {
            let v: [(f64, f64, f64); 3] = tri.map(|c| ideal(quad[c], k));
            let uv: [(f64, f64); 3] = tri.map(|c| (UV[c].0 as f64, UV[c].1 as f64));
            let area = (v[1].0 - v[0].0) * (v[2].1 - v[0].1) - (v[2].0 - v[0].0) * (v[1].1 - v[0].1);
            let (x0, x1) = (v.iter().map(|p| p.0).fold(f64::MAX, f64::min).floor() as i32, v.iter().map(|p| p.0).fold(f64::MIN, f64::max).ceil() as i32);
            let (y0, y1) = (v.iter().map(|p| p.1).fold(f64::MAX, f64::min).floor() as i32, v.iter().map(|p| p.1).fold(f64::MIN, f64::max).ceil() as i32);
            for y in y0.max(256 * (k as i32 & 1))..=y1.min(256 * (k as i32 & 1) + 239) {
                for x in x0.max(0)..=x1.min(319) {
                    let (px, py) = (x as f64, y as f64);
                    let b: [f64; 3] = std::array::from_fn(|i| {
                        let (a, c) = (v[(i + 1) % 3], v[(i + 2) % 3]);
                        ((c.0 - a.0) * (py - a.1) - (c.1 - a.1) * (px - a.0)) / area
                    });
                    let edge = |i: usize| {
                        let (a, c) = (v[(i + 1) % 3], v[(i + 2) % 3]);
                        b[i] * area.abs() / (c.0 - a.0).hypot(c.1 - a.1)
                    };
                    if (0..3).any(|i| edge(i) < 1.5) {
                        continue;
                    }
                    let w: [f64; 3] = std::array::from_fn(|i| b[i] / v[i].2);
                    let sum: f64 = w.iter().sum();
                    let (u, t) = ((0..3).map(|i| w[i] * uv[i].0).sum::<f64>() / sum, (0..3).map(|i| w[i] * uv[i].1).sum::<f64>() / sum);
                    let texel = vram(x as u32, y as u32);
                    let (du, dv) = ((texel & 0xFF) as f64 - u, ((texel >> 8) & 0x7F) as f64 - t);
                    let error = du.abs() + dv.abs();
                    sums.0 += error;
                    sums.1 += (error > 1.0) as u32 as f64;
                    sums.2 += 1.0;
                }
            }
        }
    }
}

/// The grid's edges, each by its two grid vertices, the quads on its two sides (the second none on the outline) and
/// whether the texture's u or v names the side: the twenty-four between two quads, then the outline's sixteen.
fn grid_edges() -> Vec<([usize; 2], [usize; 2], Option<[usize; 2]>, bool)> {
    let n = GRID - 1;
    let (mut inner, mut outer) = (Vec::new(), Vec::new());
    for j in 0..n {
        for i in 0..=n {
            let ends = [j * GRID + i, (j + 1) * GRID + i];
            match i {
                0 => outer.push((ends, [j, 0], None, true)),
                _ if i == n => outer.push((ends, [j, n - 1], None, true)),
                _ => inner.push((ends, [j, i - 1], Some([j, i]), true)),
            }
        }
    }
    for j in 0..=n {
        for i in 0..n {
            let ends = [j * GRID + i, j * GRID + i + 1];
            match j {
                0 => outer.push((ends, [0, i], None, false)),
                _ if j == n => outer.push((ends, [n - 1, i], None, false)),
                _ => inner.push((ends, [j - 1, i], Some([j, i]), false)),
            }
        }
    }
    inner.extend(outer);
    inner
}

/// Where frame `k`'s picture puts each of the grid's edges, against the ideal projection (§34.2): for the pixels within
/// 2.5 of an edge and 2.5 clear of its ends, those on one quad's side that the other drew less those the other way,
/// over the edge's length, which is how far the edge stands from the ideal one, in pixels, toward its second side; the
/// quad that drew a pixel is read from its texel, whose u or v is near 63 on the first side and near 0 on the second,
/// and on the outline the second side is the cleared buffer. Pixels neither drew within the grid are counted apart. At
/// a scale of N the samples are graded, each at its point (§39.1), each an N²th of a pixel.
fn edge_places(vram: impl Fn(u32, u32) -> u16, k: usize) -> (Vec<f64>, u32) {
    edge_places_at(vram, k, 1)
}

fn edge_places_at(vram: impl Fn(u32, u32) -> u16, k: usize, n: u32) -> (Vec<f64>, u32) {
    let scale = n as f64;
    const BAND: f64 = 2.5;
    let centre = |q: [usize; 2]| {
        let c: Vec<(f64, f64, f64)> = [0, 1, GRID, GRID + 1].iter().map(|d| ideal(q[0] * GRID + q[1] + d, k)).collect();
        (c.iter().map(|p| p.0).sum::<f64>() / 4.0, c.iter().map(|p| p.1).sum::<f64>() / 4.0)
    };
    let (mut places, mut holes) = (Vec::new(), 0);
    for (ends, first, second, by_u) in grid_edges() {
        let (a, b) = (ideal(ends[0], k), ideal(ends[1], k));
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let length = dx.hypot(dy);
        let side = |x: f64, y: f64| (dx * (y - a.1) - dy * (x - a.0)) / length;
        let c = centre(first);
        let toward_first = side(c.0, c.1).signum();
        let (x0, x1) = (((a.0.min(b.0) - BAND) * scale).floor() as i32, ((a.0.max(b.0) + BAND) * scale).ceil() as i32);
        let (y0, y1) = (((a.1.min(b.1) - BAND) * scale).floor() as i32, ((a.1.max(b.1) + BAND) * scale).ceil() as i32);
        let (mut moved, mut used) = (0.0, (length - 2.0 * BAND) * scale * scale);
        if used < 4.0 {
            used = f64::INFINITY;
        }
        let (top, n) = (256 * (k as i32 & 1), n as i32);
        for y in y0.max(top * n)..=y1.min((top + 240) * n - 1) {
            for x in x0.max(0)..=x1.min(320 * n - 1) {
                let (px, py) = (x as f64 / scale, y as f64 / scale);
                let along = (dx * (px - a.0) + dy * (py - a.1)) / length;
                let s = side(px, py) * toward_first;
                if along < BAND || along > length - BAND || s.abs() > BAND || s == 0.0 {
                    continue;
                }
                let texel = vram(x as u32, y as u32);
                let drawn = texel & 0x8000 != 0;
                let coordinate = if by_u { texel & 0xFF } else { texel >> 8 & 0x7F };
                // Which side drew the pixel: the first quad's texels end near 63 here and the second's start near 0.
                let by_first = match second {
                    Some(_) if !drawn => {
                        holes += 1;
                        continue;
                    }
                    Some(_) => coordinate >= 32,
                    None => drawn,
                };
                if s > 0.0 && !by_first {
                    moved -= 1.0;
                } else if s < 0.0 && by_first {
                    moved += 1.0;
                }
            }
        }
        places.push(moved / used);
    }
    (places, holes)
}

/// The edges' figures over the frames: the mean distance of an edge from its ideal place, the mean change of that
/// distance from one frame to the next (the wobble) and the worst distance, for the edges between two quads and for
/// the outline's.
fn edge_figures(frames: &[Vec<f64>]) -> [(f64, f64, f64); 2] {
    let inner = (GRID - 1) * (GRID - 2) * 2;
    [0..inner, inner..grid_edges().len()].map(|edges| {
        let (mut e, mut d, mut worst, mut n, mut nd) = (0.0, 0.0, 0.0f64, 0.0, 0.0);
        for k in 0..frames.len() {
            for i in edges.clone() {
                e += frames[k][i].abs();
                worst = worst.max(frames[k][i].abs());
                n += 1.0;
                if k > 0 {
                    d += (frames[k][i] - frames[k - 1][i]).abs();
                    nd += 1.0;
                }
            }
        }
        (e / n, d / nd, worst)
    })
}

/// §23's mesh: the grid turned 0.23° a frame for 120 frames. Its vertices' error against the ideal projection with
/// integer vertices, with MAC0's sums (§21.2's alternative) and with the precise vertices; and its texture's error
/// against the perspective-correct reference with the setting off (the software rasteriser's VRAM) and on (the
/// renderer's). The numbers are printed for Diamond_Enhancements.md §25; the test holds the improvement.
#[test]
fn the_turning_grid_shows_less_jitter_and_warp_with_the_setting_on() {
    let run = |form: Form, r: Option<Renderer>| {
        let mut m = mesh_machine();
        m.attach(Box::new(Precise::new(Precision::new(form), Tap::new(r, true))));
        let (mut off, mut on) = ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0));
        let (mut edges_off, mut edges_on, mut holes, mut last) = (Vec::new(), Vec::new(), (0, 0), 0);
        while frames_drawn(&mut m) < MESH_FRAMES && m.frames < 4 * MESH_FRAMES as i64 {
            m.advance();
            let k = frames_drawn(&mut m);
            if k == 0 || tap(&mut m).drawer.renderer.is_none() {
                continue;
            }
            let k = k - 1;
            warp(|x, y| m.board.gpu.vram.get(x, y), k, &mut off);
            let mut v = vec![0u16; 1024 * 512];
            drawn(&mut m).vram(&mut v);
            warp(|x, y| v[(y * 1024 + x) as usize], k, &mut on);
            // Each of the grid's frames once, the machine's frames between two of them showing the same one.
            if k + 1 > last {
                last = k + 1;
                let (a, b) = (edge_places(|x, y| m.board.gpu.vram.get(x, y), k), edge_places(|x, y| v[(y * 1024 + x) as usize], k));
                edges_off.push(a.0);
                edges_on.push(b.0);
                holes = (holes.0 + a.1, holes.1 + b.1);
            }
        }
        let t = tap(&mut m);
        (vertices_drawn(&t.drawer.polygons), t.precision.counts(), off, on, (edge_figures(&edges_off), edge_figures(&edges_on), holes))
    };
    let (sums, sums_counts, _, _, _) = run(Form::Sums, None);
    let Some(r) = renderer() else {
        let integer: Vec<Vec<(f64, f64)>> = sums.iter().map(|f| f.iter().map(|v| v.0).collect()).collect();
        eprintln!("integer vertices: {:?}", jitter(&integer[..MESH_FRAMES.min(integer.len())]));
        return;
    };
    let (exact, counts, off, on, (edges_off, edges_on, holes)) = run(Form::Exact, Some(r));
    assert_eq!((counts.polygons, counts.precise_polygons, counts.refused, counts.far), (counts.precise_polygons, counts.polygons, 0, 0), "every quad precise: {counts:?}");
    assert_eq!(sums_counts.precise_polygons, sums_counts.polygons);
    let frames = MESH_FRAMES.min(exact.len());
    let integer: Vec<Vec<(f64, f64)>> = exact[..frames].iter().map(|f| f.iter().map(|v| v.0).collect()).collect();
    let precise: Vec<Vec<(f64, f64)>> = exact[..frames].iter().map(|f| f.iter().map(|v| v.1.unwrap()).collect()).collect();
    let mac0: Vec<Vec<(f64, f64)>> = sums[..frames].iter().map(|f| f.iter().map(|v| v.1.unwrap()).collect()).collect();
    let (ji, jm, jp) = (jitter(&integer), jitter(&mac0), jitter(&precise));
    let (wo, wn) = ((off.0 / off.2, off.1 / off.2), (on.0 / on.2, on.1 / on.2));
    eprintln!("{frames} frames, {} vertices a frame", GRID * GRID);
    eprintln!("jitter (mean error px, mean frame-to-frame change px, worst error px): integer {ji:.4?}, MAC0's sums {jm:.4?}, precise {jp:.4?}");
    eprintln!("warp over {} pixels a frame (mean texel error, share over one texel): off {wo:.4?}, on {wn:.4?}", (off.2 / frames as f64) as u64);
    eprintln!("edges between two quads (mean distance from the ideal px, mean frame-to-frame change px, worst px): off {:.4?}, on {:.4?}; the outline's: off {:.4?}, on {:.4?}; pixels neither quad drew: off {}, on {}", edges_off[0], edges_on[0], edges_off[1], edges_on[1], holes.0, holes.1);
    assert!(jp.0 < ji.0 / 10.0 && jp.1 < ji.1 / 10.0, "precise vertices are not an order better: {jp:?} against {ji:?}");
    assert!(wn.0 < wo.0 / 2.0 && wn.1 < wo.1 / 4.0, "the texture is not nearer the reference: {wn:?} against {wo:?}");
    // §34.2: the crack fill leaves the edges two precise quads share where stage E4 drew them, and the outline no
    // farther from the ideal than the console's.
    assert!(edges_on[0].0 < 0.05 && edges_on[0].1 < 0.05 && holes.1 == 0, "the shared edges moved: {:?}, {} holes", edges_on[0], holes.1);
    assert!(edges_on[1].0 < edges_off[1].0 && edges_on[1].1 < edges_off[1].1, "the outline: {:?} against {:?}", edges_on[1], edges_off[1]);
}

/// §23's 2D program: sprites and rectangles, lines and a polyline, flat, shaded and textured polygons the processor
/// builds, a menu of quads projected by the GTE whose every vertex the processor then moves by a pixel, and a quad three
/// of whose vertices are projected and one moved: with the setting on, the picture and VRAM the software rasteriser's.
#[test]
fn a_2d_program_is_drawn_as_the_software_rasteriser_with_the_setting_on() {
    let Some(r) = renderer() else { return };
    let mut s = String::new();
    s += "lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\n";
    s += &(gp1(0) + &gp1(0x0800_0001) + &gp1(0x0300_0000));
    s += &li("t0", H);
    s += "ctc2 t0, $26\nlui t0, 160\nctc2 t0, $24\nlui t0, 120\nctc2 t0, $25\nori t0, zero, 0x1000\nctc2 t0, $0\nctc2 t0, $2\nctc2 t0, $4\nori t0, zero, 0\nctc2 t0, $5\nctc2 t0, $6\nori t0, zero, 900\nctc2 t0, $7\n";
    s += "loop:\n";
    for w in [0xE100_0508, 0xE300_0000, 0xE400_0000 | 319 | 239 << 10, 0xE500_0000, 0xE600_0000, 0x0210_2030, 0, 0x00F0_0140] {
        s += &gp0(w);
    }
    for w in [0x6000_80FF, 0x0010_0010, 0x0020_0030, 0x6580_8080, 0x0010_0050, 0x0000_1020, 0x0020_0020, 0x7D80_8080, 0x0040_0090, 0x0000_0010, 0x7480_8080, 0x0040_00A0, 0x0000_3040] {
        s += &gp0(w);
    }
    for w in [0x4000_FF00, 0x0060_0010, 0x0090_0060, 0x5000_00FF, 0x0060_0070, 0x00FF_0000, 0x0080_00C0, 0x4800_FFFF, 0x00A0_0010, 0x00B0_0030, 0x00A0_0050, 0x5555_5555] {
        s += &gp0(w);
    }
    for w in [0x2040_8020, 0x0090_00D0, 0x0080_0100, 0x00B0_0120, 0x3000_00FF, 0x00C0_0010, 0x0000_FF00, 0x00C0_0050, 0x00FF_0000, 0x00E0_0030] {
        s += &gp0(w);
    }
    for w in [0x2D80_8080, 0x00C8_0060, 0x0000_0000, 0x00C8_00A0, 0x0108_003F, 0x00E8_0060, 0x0000_3F00, 0x00E8_00A0, 0x0000_3F3F] {
        s += &gp0(w);
    }
    for (row, y) in [(0, -60i32), (1, -30)] {
        let v = |x: i32, y: i32| (x as u16 as u32 | (y as u16 as u32) << 16, 900u32);
        let (a, b, c) = (v(-100, y), v(-40, y), v(-100, y + 20));
        s += &li("t0", a.0);
        s += "mtc2 t0, $0\n";
        s += &li("t0", a.1);
        s += "mtc2 t0, $1\n";
        s += &li("t0", b.0);
        s += "mtc2 t0, $2\n";
        s += &li("t0", b.1);
        s += "mtc2 t0, $3\n";
        s += &li("t0", c.0);
        s += "mtc2 t0, $4\n";
        s += &li("t0", c.1);
        s += "mtc2 t0, $5\nnop\n.word 0x4A280030\nmfc2 t5, $12\nmfc2 t6, $13\nmfc2 t7, $14\nnop\n";
        s += if row == 0 { "addiu t5, t5, 1\naddiu t6, t6, 1\naddiu t7, t7, 1\n" } else { "addiu t7, t7, 1\n" };
        s += &gp0(if row == 0 { 0x2800_8040 } else { 0x2840_8000 });
        s += "sw t5, 0x1810(a0)\nsw t6, 0x1810(a0)\nsw t7, 0x1810(a0)\nmfc2 t8, $13\nnop\n";
        s += if row == 0 { "addiu t8, t8, 1\n" } else { "" };
        s += "sw t8, 0x1810(a0)\n";
    }
    s += "wait: lw t0, 0x1070(a0)\nnop\nandi t0, t0, 1\nbeq t0, zero, wait\nnop\n";
    s += &li("t0", 0xFFFF_FFFE);
    s += "sw t0, 0x1070(a0)\nj loop\nnop\n";
    let mut m = machine(&s);
    coordinate_texture(&mut m);
    m.attach(Box::new(Precise::new(Precision::default(), Tap::new(Some(r), false))));
    let mut c = Compared::default();
    for _ in 0..10 {
        m.advance();
        c.frame(&mut m);
    }
    c.vram(&mut m);
    let counts = tap(&mut m).precision.counts();
    eprintln!("{}; {c:?}", counts_line(&counts, Some(&drawn(&mut m).counts())));
    assert_eq!((c.pixels, c.vram, counts.precise_polygons), (0, 0, 0), "{c:?} {counts:?}");
    assert!(counts.projections >= 30 && counts.changed > 0 && counts.mixed_polygons > 0 && counts.polygons > 0, "{counts:?}");
}

// --- The crack fill (Diamond_Enhancements.md §33) ---

/// The colour the fill's programs clear their buffer to, and the halfword it is in VRAM.
const BACK: u32 = 0x0040_2010;
const BACK_HALF: u16 = 0x2082;

/// The start of the fill's programs: the GTE as the strips' test sets it, with H 1,010 over a plane 1,000 away, so
/// that a unit is 1.01 pixels and whole coordinates project between the pixels; the buffer cleared.
fn plane_program() -> String {
    let mut s = String::new();
    s += "lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\nlui s5, 0x8002\n";
    s += &(gp1(0) + &gp1(0x0800_0001) + &gp1(0x0300_0000));
    s += &li("t0", 1010);
    s += "ctc2 t0, $26\nlui t0, 160\nctc2 t0, $24\nlui t0, 120\nctc2 t0, $25\nori t0, zero, 0x1000\nctc2 t0, $0\nctc2 t0, $2\nctc2 t0, $4\nctc2 zero, $5\nctc2 zero, $6\nori t0, zero, 1000\nctc2 t0, $7\n";
    for w in [0xE100_0508, 0xE300_0000, 0xE400_0000 | 319 | 239 << 10, 0xE500_0000, 0xE600_0000, 0x0200_0000 | BACK, 0, 0x00F0_0140] {
        s += &gp0(w);
    }
    s
}

/// Three vertices on the plane into V0 to V2 and projected.
fn project3(v: [(i32, i32); 3]) -> String {
    let mut s = String::new();
    for (i, (x, y)) in v.into_iter().enumerate() {
        s += &li("t0", x as u16 as u32 | (y as u16 as u32) << 16);
        s += &format!("mtc2 t0, ${}\nmtc2 zero, ${}\n", 2 * i, 2 * i + 1);
    }
    s + "nop\n.word 0x4A280030\n"
}

/// A quad on the plane, its corners projected and sent by SWC2 straight to GP0, each with its word of `after` behind it.
fn projected_quad(command: u32, v: [(i32, i32); 4], after: Option<[u32; 4]>) -> String {
    let mut s = project3([v[0], v[1], v[2]]) + &gp0(command);
    for (i, slot) in [12, 13, 14].into_iter().enumerate() {
        s += &format!("swc2 ${slot}, 0x1810(a0)\n");
        if let Some(a) = after {
            s += &gp0(a[i]);
        }
    }
    s += &li("t0", v[3].0 as u16 as u32 | (v[3].1 as u16 as u32) << 16);
    s += "mtc2 t0, $0\nmtc2 zero, $1\nnop\n.word 0x4A180001\nswc2 $14, 0x1810(a0)\n";
    if let Some(a) = after {
        s += &gp0(a[3]);
    }
    s
}

/// A vertex word made on the processor from an x and a y in registers, and sent: no projection stands behind it.
fn built_vertex(x: &str, y: &str) -> String {
    format!("andi t6, {x}, 0xFFFF\nsll t7, {y}, 16\nor t6, t6, t7\nsw t6, 0x1810(a0)\n")
}

/// A floor's top edge from (`xl`, -32) to (`xr`, -32) projected, its two ends read back by halves, and a ring of two
/// flat quads thirty lines high built on the processor above it, meeting at the middle of the edge: the console's
/// integer vertices, a T where the two meet on the floor's edge.
fn ring(xl: i32, xr: i32) -> String {
    let mut s = project3([(xl, -32), (xr, -32), (xl, 40)]);
    s += "swc2 $12, 0(s5)\nswc2 $13, 4(s5)\nlh t1, 0(s5)\nlh t2, 2(s5)\nlh t3, 4(s5)\nnop\naddu t5, t1, t3\nsra t5, t5, 1\naddiu t8, t2, -30\n";
    for (colour, left, right) in [(0x0000_C0C0, "t1", "t5"), (0x00C0_00C0, "t5", "t3")] {
        s += &gp0(0x2800_0000 | colour);
        s += &(built_vertex(left, "t8") + &built_vertex(right, "t8") + &built_vertex(left, "t2") + &built_vertex(right, "t2"));
    }
    s
}

fn fill_machine(body: &str) -> Machine {
    let mut m = machine(&(plane_program() + body + "loop: j loop\nnop\n"));
    coordinate_texture(&mut m);
    m
}

/// The fill's program run three frames with the pair attached: the machine, and the renderer's VRAM.
fn filled(body: &str, r: Renderer) -> (Machine, Vec<u16>) {
    let mut m = fill_machine(body);
    m.attach(Box::new(Precise::new(Precision::default(), Tap::new(Some(r), true))));
    for _ in 0..3 {
        m.advance();
    }
    let mut v = vec![0u16; 1024 * 512];
    drawn(&mut m).vram(&mut v);
    (m, v)
}

/// §33's defect, held: a precise floor under a ring of integer quads that meet in a T on its top edge, the ring drawn
/// before the floor and, beside it, after. The floor's edge lies 0.68 of a pixel below the line the console draws it
/// on, so that without the fill a line of the cleared buffer shows between the two; with it no pixel the console
/// covers shows the buffer, and the pixels filled along the second floor's edge show its texture's first row, each
/// the texel of its place in the console's triangle (§33.2). The first floor is drawn twice, opaque and then
/// semi-transparent over it, as a game that shades in two passes draws: its filled pixels take both or neither, the
/// console's colour (§33.4).
#[test]
fn a_precise_floor_meets_an_integer_ring_with_no_crack() {
    let Some(r) = renderer() else { return };
    let floor = |xl: i32, xr: i32| [(xl, -32), (xr, -32), (xl, 40), (xr, 40)];
    let texture = [0, 0x0108_0078, 0x4800, 0x4878];
    let body = ring(-140, -20) + &projected_quad(0x2800_C000, floor(-140, -20), None) + &projected_quad(0x2A60_6060, floor(-140, -20), None) + &projected_quad(0x2D80_8080, floor(20, 140), Some(texture)) + &ring(20, 140);
    let (mut m, v) = filled(&body, r);
    let counts = tap(&mut m).precision.counts();
    assert_eq!((counts.polygons, counts.precise_polygons, counts.mixed_polygons), (7, 3, 0), "three floors precise, four ring quads not: {counts:?}");
    let (mut covered, mut cracks) = (0, 0);
    for y in 0..240u32 {
        for x in 0..320u32 {
            if m.board.gpu.vram.get(x, y) != BACK_HALF {
                covered += 1;
                cracks += (v[(y * 1024 + x) as usize] == BACK_HALF) as u32;
            }
        }
    }
    // The second floor's top row as the console draws it, 87, from its corner at 180 to the one at 301: its corners
    // project to x 180.2 and 301.4, y 87.68.
    let (mut filled, mut off) = (0, 0);
    for x in 181..301u32 {
        let t = v[(87 * 1024 + x) as usize];
        let u = (x as f64 - 180.0) * 120.0 / 121.0;
        filled += (t & 0x8000 != 0) as u32;
        off += (t & 0x8000 == 0 || t >> 8 & 0x7F != 0 || ((t & 0xFF) as f64 - u).abs() > 0.75) as u32;
    }
    // The first floor's top row, 87 from 18 to 138, all of it filled: the console's own colour, both passes.
    let once = (18..139u32).filter(|&x| v[(87 * 1024 + x) as usize] != m.board.gpu.vram.get(x, 87)).count();
    eprintln!("{covered} pixels the console covers, {cracks} of them the cleared buffer in the renderer's picture; {filled} of the textured floor's first row filled, {off} off their texel; {once} of the twice-drawn floor's first row not the console's colour");
    assert!(covered > 20_000 && cracks == 0, "{cracks} pixels of the cleared buffer show where the console draws");
    assert_eq!((filled, off, once), (120, 0, 0), "the filled rows");
}

/// Two precise quads sharing a slanted edge, the left drawn first and, beside them, the right: what the later spills
/// over the earlier. Each pixel of the rows the shared edge crosses belongs to the quad whose side of the precise edge
/// its centre is on, as stage E4 drew it; a fill that took every pixel of the console's own triangle would give the
/// later quad the pixels between the console's edge and the precise one (§33.3).
#[test]
fn two_precise_quads_sharing_an_edge_spill_nothing() {
    let Some(r) = renderer() else { return };
    let pair = |x: i32| ([(x - 60, -40), (x, -40), (x - 70, 40), (x - 10, 40)], [(x, -40), (x + 60, -40), (x - 10, 40), (x + 50, 40)]);
    let (a, b) = (pair(-70), pair(90));
    let body = projected_quad(0x2800_00E0, a.0, None) + &projected_quad(0x28E0_0000, a.1, None) + &projected_quad(0x28E0_0000, b.1, None) + &projected_quad(0x2800_00E0, b.0, None);
    let (mut m, v) = filled(&body, r);
    let t = tap(&mut m);
    assert_eq!(t.precision.counts().precise_polygons, 4);
    let (left, right) = (0x001Cu16, 0x7000u16);
    let (mut spilled, mut graded) = (0, 0);
    for first in [0, 2] {
        // The shared edge by the precise vertices the drawer was given: the left quad's second and fourth.
        let (words, offset, precise) = &t.drawer.polygons[if first == 0 { 0 } else { 3 }];
        let (q, f) = (decode::polygon(words, *offset), precise.unwrap());
        let at = |c: usize| (q.v[c].x as f64 + f[c].dx as f64 / 256.0, q.v[c].y as f64 + f[c].dy as f64 / 256.0);
        let (top, bottom) = (at(1), at(3));
        for y in top.1.ceil() as u32 + 2..bottom.1.floor() as u32 - 2 {
            let edge = top.0 + (bottom.0 - top.0) * (y as f64 - top.1) / (bottom.1 - top.1);
            for x in edge.floor() as u32 - 6..edge.floor() as u32 + 8 {
                if (x as f64 - edge).abs() < 0.01 {
                    continue;
                }
                graded += 1;
                spilled += (v[(y * 1024 + x) as usize] != if (x as f64) < edge { left } else { right }) as u32;
            }
        }
    }
    eprintln!("{graded} pixels about the two shared edges graded, {spilled} not their quad's");
    assert!(graded > 1500 && spilled == 0, "{spilled} pixels spilled over a shared edge");
}

/// Semi-transparent quads are blended once: a flat pair sharing a slanted edge, and a textured pair every texel of
/// which is semi-transparent, each over the cleared buffer. Every pixel is the buffer's or one quad's blend with it;
/// none is a blend of a blend, which a fill of a semi-transparent pixel would make where two meet (§33.4).
#[test]
fn a_semi_transparent_pair_is_blended_once() {
    let Some(r) = renderer() else { return };
    let pair = |x: i32| ([(x - 60, -40), (x, -40), (x - 70, 40), (x - 10, 40)], [(x, -40), (x + 60, -40), (x - 10, 40), (x + 50, 40)]);
    let (a, b) = (pair(-70), pair(90));
    let texture = [0, 0x0109_003F, 0x3F00, 0x3F3F];
    let body = projected_quad(0x2A00_00E0, a.0, None) + &projected_quad(0x2AE0_0000, a.1, None) + &projected_quad(0x2F80_8080, b.0, Some(texture)) + &projected_quad(0x2F80_8080, b.1, Some(texture));
    let mut m = fill_machine(&body);
    // A texture page at (576, 0) of one semi-transparent colour.
    let texel = 0x8000 | 0x0380;
    for y in 0..64 {
        for x in 0..64 {
            m.board.gpu.vram.set(576 + x, y, texel);
        }
    }
    m.attach(Box::new(Precise::new(Precision::default(), Tap::new(Some(r), false))));
    for _ in 0..3 {
        m.advance();
    }
    assert_eq!(tap(&mut m).precision.counts().precise_polygons, 4);
    let mut v = vec![0u16; 1024 * 512];
    drawn(&mut m).vram(&mut v);
    let half = |b: u16, f: u16| (0..3).map(|c| (((b >> (5 * c) & 31) + (f >> (5 * c) & 31)) >> 1) << (5 * c)).sum::<u16>();
    let once = [BACK_HALF, half(BACK_HALF, 0x001C), half(BACK_HALF, 0x7000), half(BACK_HALF, texel & 0x7FFF) | 0x8000];
    let (mut blended, mut other) = (0, 0);
    for y in 60..180u32 {
        for x in 0..320u32 {
            let p = v[(y * 1024 + x) as usize];
            blended += (p != BACK_HALF) as u32;
            other += !once.contains(&p) as u32;
        }
    }
    eprintln!("{blended} pixels blended, {other} neither the buffer nor one blend with it");
    assert!(blended > 15_000 && other == 0, "{other} pixels blended twice");
}

fn saved(m: &mut Machine) -> Vec<u8> {
    m.settle();
    let mut s = vec![0; m.state_size()];
    let n = m.save_state(&mut s).unwrap();
    s.truncate(n);
    s
}

fn paired() -> Option<Box<Precise<Tap>>> {
    Some(Box::new(Precise::new(Precision::default(), Tap::new(Some(renderer()?), false))))
}

/// §33.5: the marks are the state's. Two precise quads that share a slanted edge, the left drawn at once and the right
/// three vertical blanks later, and a state saved between the two. The machine that saved it and one that loads it
/// draw the right quad alike, to the pixel; one given the same state with its marks cleared fills the right quad over
/// the left's pixels along their edge, as the fill without marks does (§33.3), so the block needs them.
#[test]
fn a_state_between_two_quads_that_share_an_edge_keeps_their_marks() {
    if renderer().is_none() {
        return;
    }
    let (left, right) = ([(-130, -40), (-70, -40), (-140, 40), (-80, 40)], [(-70, -40), (-10, -40), (-80, 40), (-20, 40)]);
    let acknowledged = li("t0", 0xFFFF_FFFE) + "sw t0, 0x1070(a0)\n";
    let mut body = projected_quad(0x2800_00E0, left, None) + &acknowledged;
    for n in 0..3 {
        body += &format!("blank{n}: lw t0, 0x1070(a0)\nnop\nandi t0, t0, 1\nbeq t0, zero, blank{n}\nnop\n");
        body += &acknowledged;
    }
    body += &projected_quad(0x28E0_0000, right, None);
    let loaded = |state: &[u8]| {
        let mut m = fill_machine(&body);
        m.attach(paired().unwrap());
        m.load_state(state).unwrap();
        for _ in 0..3 {
            m.advance();
        }
        (tap(&mut m).precision.counts().precise_polygons, drawn(&mut m).picture().0.to_vec())
    };
    let mut a = fill_machine(&body);
    a.attach(paired().unwrap());
    a.advance();
    a.advance();
    assert_eq!(tap(&mut a).precision.counts().precise_polygons, 1, "the left quad alone before the state");
    let state = saved(&mut a);
    for _ in 0..3 {
        a.advance();
    }
    assert_eq!(tap(&mut a).precision.counts().precise_polygons, 2, "the right quad after it");
    let picture = drawn(&mut a).picture().0.to_vec();
    let (_, continued) = loaded(&state);
    let differing = |other: &[u8]| picture.chunks(4).zip(other.chunks(4)).filter(|(x, y)| x != y).count();
    assert_eq!(differing(&continued), 0, "the machine that loaded the state draws another picture");
    let layout = a.layout();
    let marks: Vec<usize> = layout.lines().find(|l| l.ends_with("Marks")).expect("the marks' field").split_whitespace().take(2).map(|n| n.parse().unwrap()).collect();
    let mut cleared = state.clone();
    cleared[marks[0]..marks[0] + marks[1]].fill(0);
    let (_, unmarked) = loaded(&cleared);
    eprintln!("with the marks cleared from the state {} pixels differ", differing(&unmarked));
    assert!(differing(&unmarked) > 0, "the marks made no difference: the test no longer holds them");
}

/// §24.4 on the grid: a state saved with the setting on, loaded on a machine with it on, continues the picture as the
/// machine that saved it draws it, frame for frame; saved, loaded and saved again it is the same bytes; loaded with the
/// setting off the machine continues as the first; and a state made with it off loads with it on and continues the
/// machine, its polygons precise again within two frames.
#[test]
fn a_state_with_the_setting_on_continues_its_picture_and_loads_both_ways() {
    let Some(pair) = paired() else { return };
    let mut a = mesh_machine();
    a.attach(pair);
    for _ in 0..40 {
        a.advance();
    }
    let state = saved(&mut a);
    a.load_state(&state).unwrap();
    assert!(saved(&mut a) == state, "save, load, save gives other bytes");
    let mut b = mesh_machine();
    b.attach(paired().unwrap());
    b.load_state(&state).unwrap();
    let mut c = mesh_machine();
    c.load_state(&state).unwrap();
    for f in 0..20 {
        for m in [&mut a, &mut b, &mut c] {
            m.advance();
        }
        let (pa, pb) = (drawn(&mut a).picture().0.to_vec(), drawn(&mut b).picture().0.to_vec());
        assert!(pa == pb, "frame {f} after the load: the pictures differ in {} pixels", pa.chunks(4).zip(pb.chunks(4)).filter(|(x, y)| x != y).count());
    }
    assert_eq!((state_digest(&a), state_digest(&b)), (state_digest(&c), state_digest(&c)), "the machine continues the same, the layer on or off");
    assert!(drawn(&mut a).counts().precise > 0);
    let plain = saved(&mut c);
    let mut d = mesh_machine();
    d.attach(paired().unwrap());
    d.load_state(&plain).unwrap();
    let before = tap(&mut d).precision.counts().precise_polygons;
    for _ in 0..2 {
        d.advance();
        c.advance();
    }
    assert!(tap(&mut d).precision.counts().precise_polygons > before && state_digest(&d) == state_digest(&c), "a state without the blocks: precise again, and the machine the same");
    assert!(state.len() > plain.len() + 4_000_000, "{} against {}", state.len(), plain.len());
}

/// §26.3's defect, held: four strips of a picture side by side at one depth, each its texture's 64 columns at a little
/// over one texel a pixel, their corners projected by the GTE to fractions of a pixel and sent by SWC2 straight to GP0.
/// With sub-pixel corners the last column of a strip lies within a fraction of a pixel of its right edge, where the
/// console's rounding of the texel gives column 64, the next region's; held short of it, no pixel of the strips shows a
/// texel past its strip, and none the software rasteriser draws is left out: the strips' first rows and the first
/// strip's first column, which the console's own triangles cover and the precise ones do not, are filled (§33).
#[test]
fn a_strip_of_a_picture_shows_no_texel_past_its_edge() {
    let Some(r) = renderer() else { return };
    let mut s = String::new();
    s += "lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\n";
    s += &(gp1(0) + &gp1(0x0800_0001) + &gp1(0x0300_0000));
    s += &li("t0", 1001);
    s += "ctc2 t0, $26\nlui t0, 160\nctc2 t0, $24\nlui t0, 120\nctc2 t0, $25\nori t0, zero, 0x1000\nctc2 t0, $0\nctc2 t0, $2\nctc2 t0, $4\nctc2 zero, $5\nctc2 zero, $6\nori t0, zero, 1000\nctc2 t0, $7\n";
    for w in [0xE100_0508, 0xE300_0000, 0xE400_0000 | 319 | 239 << 10, 0xE500_0000, 0xE600_0000, 0x0200_0000, 0, 0x00F0_0140] {
        s += &gp0(w);
    }
    let v = |x: i32, y: i32| (x as u16 as u32 | (y as u16 as u32) << 16, 0u32);
    for k in 0..4 {
        let (x0, x1) = (-128 + 64 * k, -64 + 64 * k);
        for (i, (x, y)) in [(x0, -32), (x1, -32), (x0, 32)].into_iter().enumerate() {
            s += &li("t0", v(x, y).0);
            s += &format!("mtc2 t0, ${}\nmtc2 zero, ${}\n", 2 * i, 2 * i + 1);
        }
        s += "nop\n.word 0x4A280030\n";
        s += &gp0(0x2D80_8080);
        s += "swc2 $12, 0x1810(a0)\n";
        s += &gp0(0);
        s += "swc2 $13, 0x1810(a0)\n";
        s += &gp0(0x0108_0040);
        s += "swc2 $14, 0x1810(a0)\n";
        s += &gp0(0x3F00);
        s += &li("t0", v(x1, 32).0);
        s += "mtc2 t0, $0\nmtc2 zero, $1\nnop\n.word 0x4A180001\nswc2 $14, 0x1810(a0)\n";
        s += &gp0(0x3F40);
    }
    s += "loop: j loop\nnop\n";
    let mut m = machine(&s);
    coordinate_texture(&mut m);
    m.attach(Box::new(Precise::new(Precision::default(), Tap::new(Some(r), false))));
    for _ in 0..3 {
        m.advance();
    }
    let counts = tap(&mut m).precision.counts();
    assert_eq!(counts.precise_polygons, 4, "{counts:?}");
    let mut v = vec![0u16; 1024 * 512];
    drawn(&mut m).vram(&mut v);
    let (mut past, mut drawn_pixels) = (0u32, 0u32);
    for y in 80..160u32 {
        for x in 20..300u32 {
            let t = v[(y * 1024 + x) as usize];
            if t & 0x8000 != 0 {
                drawn_pixels += 1;
                past += (t & 0xFF >= 64) as u32;
            }
        }
    }
    let software: Vec<(u32, u32)> = (80..160u32).flat_map(|y| (20..300u32).map(move |x| (x, y))).filter(|&(x, y)| m.board.gpu.vram.get(x, y) & 0x8000 != 0).collect();
    let missing = software.iter().filter(|&&(x, y)| v[(y * 1024 + x) as usize] & 0x8000 == 0).count();
    eprintln!("{drawn_pixels} pixels of the strips drawn, the software rasteriser's {} and {missing} of those not; {past} show a texel past their strip", software.len());
    assert!(past == 0 && missing == 0 && drawn_pixels as usize - software.len() <= 64 * 6, "{past} pixels past the strips' texels, {missing} left out");
}

// --- The crack fill at N x (Diamond_Enhancements.md §39.7) ---

/// A fill program run three frames with the renderer at `scale`, paired with the precision layer or alone (the
/// console's own polygons at that scale); the machine and every sample of the renderer's mirror.
fn filled_at(body: &str, scale: u32, paired: bool) -> Option<(Machine, Vec<u32>)> {
    let r = Renderer::with(None, diamond_renderer::Options { scale, ..Default::default() }).ok()?.keeping(true);
    let mut m = fill_machine(body);
    if paired {
        m.attach(Box::new(Precise::new(Precision::default(), Tap::new(Some(r), true))));
    } else {
        m.attach(Box::new(r));
    }
    for _ in 0..3 {
        m.advance();
    }
    let layer: &mut dyn Any = m.layer().expect("attached");
    let samples = match layer.downcast_mut::<Precise<Tap>>() {
        Some(p) => p.drawer.renderer.as_mut().expect("a renderer").samples().to_vec(),
        None => layer.downcast_mut::<Renderer>().expect("the renderer").samples().to_vec(),
    };
    Some((m, samples))
}

/// §33's floor and ring at 2x and 4x: no sample the console's own polygons cover at that scale shows the cleared
/// buffer with precise geometry on; and two precise quads sharing a slanted edge keep it where they draw it, sample by
/// sample, each sample about the edge its own quad's by the side of the precise edge its point lies on.
#[test]
fn at_n_the_fill_closes_every_crack_and_spills_over_no_shared_edge() {
    if renderer().is_none() {
        return;
    }
    let floor = |xl: i32, xr: i32| [(xl, -32), (xr, -32), (xl, 40), (xr, 40)];
    let texture = [0, 0x0108_0078, 0x4800, 0x4878];
    let rings = ring(-140, -20) + &projected_quad(0x2800_C000, floor(-140, -20), None) + &projected_quad(0x2A60_6060, floor(-140, -20), None) + &projected_quad(0x2D80_8080, floor(20, 140), Some(texture)) + &ring(20, 140);
    let pair = |x: i32| ([(x - 60, -40), (x, -40), (x - 70, 40), (x - 10, 40)], [(x, -40), (x + 60, -40), (x - 10, 40), (x + 50, 40)]);
    let (a, b) = (pair(-70), pair(90));
    let quads = projected_quad(0x2800_00E0, a.0, None) + &projected_quad(0x28E0_0000, a.1, None) + &projected_quad(0x28E0_0000, b.1, None) + &projected_quad(0x2800_00E0, b.0, None);
    for scale in [2u32, 4] {
        let n = scale as usize;
        let (_, console) = filled_at(&rings, scale, false).unwrap();
        let (_, precise) = filled_at(&rings, scale, true).unwrap();
        let (mut covered, mut cracks) = (0, 0);
        for y in 0..240 * n {
            for x in 0..320 * n {
                let at = y * 1024 * n + x;
                if console[at] as u16 != BACK_HALF {
                    covered += 1;
                    cracks += (precise[at] as u16 == BACK_HALF) as u32;
                }
            }
        }
        let (mut m, v) = filled_at(&quads, scale, true).unwrap();
        let t = tap(&mut m);
        let (left, right) = (0x001Cu16, 0x7000u16);
        let (mut spilled, mut graded) = (0, 0);
        for first in [0, 2] {
            let (words, offset, precise) = &t.drawer.polygons[if first == 0 { 0 } else { 3 }];
            let (q, f) = (decode::polygon(words, *offset), precise.unwrap());
            let at = |c: usize| (q.v[c].x as f64 + f[c].dx as f64 / 256.0, q.v[c].y as f64 + f[c].dy as f64 / 256.0);
            let (top, bottom) = (at(1), at(3));
            for sy in (top.1.ceil() as usize + 2) * n..(bottom.1.floor() as usize - 2) * n {
                let y = sy as f64 / scale as f64;
                let edge = top.0 + (bottom.0 - top.0) * (y - top.1) / (bottom.1 - top.1);
                let e = (edge * scale as f64).floor() as usize;
                for sx in e - 6 * n..e + 8 * n {
                    let x = sx as f64 / scale as f64;
                    if (x - edge).abs() < 0.01 {
                        continue;
                    }
                    graded += 1;
                    spilled += (v[sy * 1024 * n + sx] as u16 != if x < edge { left } else { right }) as u32;
                }
            }
        }
        eprintln!("{scale}x: {covered} samples the console's polygons cover, {cracks} of them the cleared buffer with precise geometry; {graded} samples about the shared edges, {spilled} not their quad's");
        assert_eq!((cracks, spilled), (0, 0), "at {scale}x");
        assert!(covered > 20_000 * n * n && graded > 1500 * n * n);
    }
}

/// §34.2's figures at 2x and 4x: the turning grid with precise geometry at the scale, its edges graded sample by sample.
/// The edges two precise quads share stay where the precise vertices put them, as at 1x, and no sample between two
/// quads is left undrawn; the outline's figures are printed beside them.
#[test]
fn the_turning_grids_shared_edges_hold_at_n() {
    if renderer().is_none() {
        return;
    }
    for scale in [2u32, 4] {
        let r = Renderer::with(None, diamond_renderer::Options { scale, ..Default::default() }).unwrap().keeping(true);
        let mut m = mesh_machine();
        m.attach(Box::new(Precise::new(Precision::default(), Tap::new(Some(r), false))));
        let (mut edges, mut holes, mut last) = (Vec::new(), 0, 0);
        while frames_drawn(&mut m) < MESH_FRAMES && m.frames < 4 * MESH_FRAMES as i64 {
            m.advance();
            let k = frames_drawn(&mut m);
            if k == 0 || k <= last {
                continue;
            }
            last = k;
            let wide = 1024 * scale;
            let s = drawn(&mut m).samples().to_vec();
            let (places, h) = edge_places_at(|x, y| s[(y * wide + x) as usize] as u16, k - 1, scale);
            edges.push(places);
            holes += h;
        }
        let [inner, outline] = edge_figures(&edges);
        eprintln!("{scale}x, {} frames: edges between two quads {inner:.4?}, the outline {outline:.4?} (mean distance from the ideal px, mean change px, worst px); {holes} samples neither quad drew", edges.len());
        assert!(inner.0 < 0.05 && inner.1 < 0.05 && holes == 0, "the shared edges at {scale}x: {inner:?}, {holes} holes");
    }
}
