//! Stage E3's gate (Diamond_Enhancements.md §17): the Vulkan renderer attached from power-on, its picture against the
//! software rasteriser's displayed picture at every frame compared, and its VRAM against the GPU's, on the runs the
//! goldens pin: the project's programs, ps1-tests' release and the bench discs along their anchors; and the goldens
//! themselves with the renderer attached. A machine without a Vulkan device stands each test down with a line.

use std::any::Any;

use diamond::machine::Machine;
use diamond_renderer::Renderer;

#[path = "support/goldens.rs"]
mod goldens;
use goldens::*;

#[path = "support/records.rs"]
mod records;

/// A renderer on the device `EMUSEN_GPU_DEVICE` names.
fn renderer() -> Renderer {
    Renderer::new(None).expect("Vulkan hardware device required")
}

fn attached(m: &mut Machine) -> &mut Renderer {
    let layer: &mut dyn Any = m.layer().expect("the renderer attached");
    layer.downcast_mut::<Renderer>().expect("the renderer")
}

/// How the renderer's picture and VRAM stood against the machine's over a run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Compared {
    frames: u64,
    /// Frames whose pictures differed, the pixels that did, and the first frame that did.
    frames_differing: u64,
    pixels: u64,
    first: Option<i64>,
    /// VRAM's halfwords that differed at the end.
    vram: u64,
    first_vram: Option<(u32, u32, u16, u16)>,
}

impl Compared {
    fn frame(&mut self, m: &mut Machine) {
        let frames = m.frames;
        let made: Vec<u8> = m.board.gpu.display.frame().to_vec();
        let (w, h) = (m.board.gpu.display.width, m.board.gpu.display.height);
        let (drawn, rw, rh) = attached(m).picture();
        self.frames += 1;
        let differing = if (rw, rh) != (w, h) { (w * h) as u64 } else { made.chunks(4).zip(drawn.chunks(4)).filter(|(a, b)| a != b).count() as u64 };
        if differing > 0 {
            self.frames_differing += 1;
            self.pixels += differing;
            self.first.get_or_insert(frames);
        }
    }

    fn vram(&mut self, m: &mut Machine) {
        let mut drawn = vec![0u16; 1024 * 512];
        attached(m).vram(&mut drawn);
        for (i, &d) in drawn.iter().enumerate() {
            let (x, y) = ((i % 1024) as u32, (i / 1024) as u32);
            let v = m.board.gpu.vram.get(x, y);
            if v != d {
                self.vram += 1;
                self.first_vram.get_or_insert((x, y, v, d));
            }
        }
    }
}

/// `m` run `frames` frames with the renderer attached from the start, the pictures compared at every frame and VRAM at
/// the end; and the run's golden digests, taken with the renderer attached, which keeps no block in the state.
fn compare(mut m: Machine, r: Renderer, frames: usize) -> (Compared, diamond_renderer::Counts, String) {
    m.attach(Box::new(r));
    let mut c = Compared::default();
    let mut fold = Fold::new();
    for _ in 0..frames {
        m.advance();
        fold.frame(&mut m);
        c.frame(&mut m);
    }
    c.vram(&mut m);
    let counts = attached(&mut m).counts();
    (c, counts, end_digests(&m, &fold))
}

/// The goldens' lines a run made against the file's, each that differs named.
fn goldens_equal(file: &str, made: &[String]) -> Vec<String> {
    let kept = kept_goldens(file);
    let mut differ: Vec<String> = made.iter().zip(&kept).filter(|(a, b)| a != b).map(|(a, b)| format!("  made {a}\n  kept {b}")).collect();
    if made.len() != kept.len() {
        differ.push(format!("  {} lines made, {} kept", made.len(), kept.len()));
    }
    differ
}

/// The halfwords of VRAM the renderer draws otherwise than the GPU, by program, each the texture cache's: the GPU's cache
/// holds texels VRAM no longer does, which the renderer, reading VRAM as it stands, cannot (Diamond_Enhancements.md §17.2).
/// A build whose cache always misses draws both as the renderer does.
const TEXTURE_CACHE_HALFWORDS: [(&str, u64); 2] = [("texture-cache.s", 128), ("triangle-rows.s", 3680)];

fn line(name: &str, c: &Compared, counts: &diamond_renderer::Counts) -> String {
    format!("{name}: {} frames, {} differing ({} pixels, first {:?}), VRAM {} ({:?}); {} records, {} stretches, {} copies, {} feedback", c.frames, c.frames_differing, c.pixels, c.first, c.vram, c.first_vram, counts.records, counts.stretches, counts.copies, counts.feedback)
}

mod vulkan_gpu {
    use super::*;

    /// The project's programs for 60 frames: renderer picture equals displayed picture and VRAM matches GS outside texcache.
    #[test]
    fn the_renderer_draws_the_projects_programs_as_the_software_rasteriser() {
        let r = renderer();
        eprintln!("{}", r.report());
        drop(r);
        let (mut failed, mut goldens) = (Vec::new(), Vec::new());
        for (name, source) in own_programs() {
            let r = renderer();
            let (c, counts, digests) = compare(golden_program(&name, &source), r, GOLDEN_PROGRAM_FRAMES);
            eprintln!("{}", line(&name, &c, &counts));
            let cache = TEXTURE_CACHE_HALFWORDS.iter().find(|p| p.0 == name).map_or(0, |p| p.1);
            if c.pixels != 0 || c.vram != cache {
                failed.push(line(&name, &c, &counts));
            }
            goldens.push(format!("{name}\t{GOLDEN_PROGRAM_FRAMES}\t{digests}"));
        }
        let differ = goldens_equal("programs.txt", &goldens);
        assert!(failed.is_empty() && differ.is_empty(), "differing:\n{}\ngoldens with the renderer attached:\n{}", failed.join("\n"), differ.join("\n"));
    }

    /// ps1-tests' release on the open kernel, every program for the frames its golden runs.
    #[test]
    #[ignore = "needs EMUSEN_DIAMOND_PS1TESTS corpus"]
    fn the_renderer_draws_ps1_tests_as_the_software_rasteriser() {
        let corpus = records::corpus().unwrap_or_else(|| panic!("{} corpus required", records::CORPUS_VARIABLE));
        let (mut failed, mut goldens) = (Vec::new(), Vec::new());
        for path in ps1_tests_programs(&corpus) {
            let r = renderer();
            let (frames, market) = ps1_tests_run(&path);
            let (c, counts, digests) = compare(ps1_tests_machine(&corpus, &path, market), r, frames);
            eprintln!("{}", line(&path, &c, &counts));
            if c.pixels != 0 || c.vram != 0 {
                failed.push(line(&path, &c, &counts));
            }
            goldens.push(format!("{path}\t{frames}\t{digests}"));
        }
        let differ = goldens_equal("ps1-tests.txt", &goldens);
        assert!(failed.is_empty() && differ.is_empty(), "differing:\n{}\ngoldens with the renderer attached:\n{}", failed.join("\n"), differ.join("\n"));
    }

    /// The bench discs, each played by its press script to frame 3,000 as its goldens are.
    #[test]
    #[ignore = "needs EMUSEN_DIAMOND_DISCS corpus"]
    fn the_renderer_draws_the_bench_discs_as_the_software_rasteriser() {
        let folder = std::env::var_os("EMUSEN_DIAMOND_DISCS").expect("EMUSEN_DIAMOND_DISCS corpus folder required");
        let folder = std::path::PathBuf::from(folder);
        let (mut failed, mut goldens) = (Vec::new(), Vec::new());
        for (file, script) in GOLDEN_DISCS {
            let r = renderer();
            let (anchor, presses, sticks) = press_script(script);
            goldens.push(format!("{file}\t{script}\tanchor {anchor}"));
            let mut m = disc_maker(folder.join(file))();
            m.insert_card(0, None);
            m.attach(Box::new(r));
            let (mut c, mut fold) = (Compared::default(), Fold::new());
            let mut anchors = Vec::new();
            while m.frames < GOLDEN_DISC_FRAMES {
                press(&mut m, &presses, &sticks);
                m.advance();
                fold.frame(&mut m);
                c.frame(&mut m);
                if m.frames % GOLDEN_DISC_ANCHOR == 0 {
                    let before = c.vram;
                    c.vram(&mut m);
                    anchors.push((m.frames, c.vram - before));
                    goldens.push(format!("\t{}\tpicture {:016X}\tsound {:016X}\tstate {:016X}", m.frames, fold.picture, fold.sound, state_digest(&m)));
                }
            }
            let counts = attached(&mut m).counts();
            eprintln!("{}; VRAM by anchor {:?}", line(file, &c, &counts), anchors.iter().filter(|a| a.1 > 0).collect::<Vec<_>>());
            if c.pixels != 0 || c.vram != 0 {
                failed.push(line(file, &c, &counts));
            }
        }
        let differ = goldens_equal("discs.txt", &goldens);
        assert!(failed.is_empty() && differ.is_empty(), "differing:\n{}\ngoldens with the renderer attached:\n{}", failed.join("\n"), differ.join("\n"));
    }
}
