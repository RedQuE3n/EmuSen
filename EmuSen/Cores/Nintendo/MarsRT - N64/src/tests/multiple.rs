//! The picture drawn at a multiple beside the machine's own: C#'s tests of Mars_Rdp.md §11 on MarsRT, the multiple's threads, and its
//! scan-out and average. See Mars_Native.md §6.4.

use super::threads::{FRAMEBUFFER, LIST, ROWS, WIDTH, at_once, hand_over, hazards, scene, shaded, state};
use crate::ffi::Core;
use crate::machine::Machine;
use crate::vi::scan::{RASTER_WIDTH, Scanout, box_average};

fn at(scale: i32, workers: usize) -> Machine {
    let mut m = at_once();
    m.set_scale(scale);
    if workers > 1 {
        m.set_verify_rdp(true);
        m.set_rdp_workers(workers);
        m.set_threaded_rdp(true);
    }
    m
}

fn run(m: &mut Machine, list: &[u64]) {
    hand_over(m, list, LIST);
    m.join_rdp();
}

/// The last is the list a split draws parts of alone, which the processors at the multiple must follow the native one in (Mars_Native.md §6.4).
fn lists() -> [Vec<u64>; 5] {
    [scene(0x1234_1234, false, 0, false), shaded(0x5566_7788, true, false, false), shaded(0x99AA_BBCC, true, true, false), hazards(), past_the_width()]
}

/// Fill rectangles past the image's right edge under a scissor wider than it: drawn alone at one, by the rectangle's own edge, which is the
/// console's even at a multiple, so only the native processor's decision is right for the processor at the multiple.
fn past_the_width() -> Vec<u64> {
    let mut list = vec![(0x2F << 56) | (3 << 52), (0x3F << 56) | (2 << 51) | (((WIDTH - 1) as u64) << 32) | FRAMEBUFFER as u64, (0x2D << 56) | ((((WIDTH + 16) << 2) as u64) << 12) | ((ROWS << 2) as u64)];
    for band in 0..12u64 {
        list.push((0x37 << 56) | (0x1357_2468u64.wrapping_mul(band + 1) & 0xFFFF_FFFF));
        list.push((0x36 << 56) | ((((WIDTH + 8) << 2) as u64) << 44) | (((band * 20 + 19) << 2) << 32) | ((band * 7) << 2 << 12) | ((band * 20) << 2));
    }
    list.push(0x29 << 56);
    list
}

/// `Drawing_at_a_multiple_leaves_the_machines_memory_as_at_one`: the machine's RDRAM, hidden bits and state are those of the machine at one.
#[test]
fn drawing_at_a_multiple_leaves_the_machine_s_memory_and_state_as_at_one() {
    for (scale, workers) in [(2, 1), (2, 3), (3, 2), (4, 4)] {
        for list in lists() {
            let mut one = at(1, 1);
            let mut scaled = at(scale, workers);
            run(&mut one, &list);
            run(&mut scaled, &list);
            assert!(one.bus.rdram == scaled.bus.rdram, "{scale} {workers}: RDRAM");
            assert!(one.bus.rdram_hidden == scaled.bus.rdram_hidden, "{scale} {workers}: the hidden bits");
            assert!(state(&one) == state(&scaled), "{scale} {workers}: the state");
            assert!(scaled.bus.dp.scaled_drawn());
            assert!(!one.bus.dp.scaled_drawn());
        }
    }
}

/// `A_fill_at_a_multiple_is_the_fill_at_one_at_every_pixel_of_the_multiple`, at two, three and four.
#[test]
fn a_fill_at_a_multiple_is_the_fill_at_one_at_every_pixel_of_the_multiple() {
    for scale in [2usize, 3, 4] {
        let mut one = at(1, 1);
        let mut scaled = at(scale as i32, 1);
        run(&mut one, &scene(0x1234_1234, false, 0, false));
        run(&mut scaled, &scene(0x1234_1234, false, 0, false));
        let wide = &scaled.bus.dp.multiple.rdram;
        assert_eq!(wide.len(), one.bus.rdram.len() * scale * scale);
        let image = FRAMEBUFFER as usize * scale * scale;
        let (width, rows) = (WIDTH as usize, ROWS as usize);
        for y in 0..rows {
            for x in 0..width {
                let native = FRAMEBUFFER as usize + (y * width + x) * 2;
                for j in 0..scale {
                    for i in 0..scale {
                        let at = image + ((y * scale + j) * width * scale + x * scale + i) * 2;
                        assert!(one.bus.rdram[native] == wide[at] && one.bus.rdram[native + 1] == wide[at + 1], "{scale}: pixel {x},{y} of the multiple {i},{j} differs");
                    }
                }
            }
        }
        assert!(one.bus.rdram[FRAMEBUFFER as usize + 1] != 0, "the console's fill drew nothing");
    }
}

/// The multiple drawn by several processors is the multiple drawn by one, byte for byte over the whole shadow: the split is exact at a multiple as at one.
#[test]
fn the_multiple_drawn_by_several_processors_is_the_multiple_drawn_by_one() {
    for scale in [2, 4] {
        for workers in [2usize, 3, 4] {
            for list in lists() {
                let mut alone = at(scale, 1);
                let mut shared = at(scale, workers);
                run(&mut alone, &list);
                run(&mut shared, &list);
                let (a, s) = (&alone.bus.dp.multiple.0, &shared.bus.dp.multiple.0);
                assert!(a.rdram == s.rdram, "{scale} {workers}: the shadow");
                assert!(a.hidden == s.hidden, "{scale} {workers}: the shadow's hidden bits");
                assert!(a.rdram.iter().any(|&b| b != 0), "nothing was drawn at the multiple");
            }
        }
    }
}

/// `After_a_state_is_read_the_multiple_is_drawn_and_shown_again`: a state read empties the shadow, and the next draw shows it again, on the threads.
#[test]
fn after_a_state_is_read_the_multiple_is_drawn_and_shown_again() {
    let mut m = at(2, 3);
    run(&mut m, &scene(0x1234_1234, false, 0, false));
    assert!(m.bus.dp.scaled_drawn());
    let snapshot = m.save_state_vec(true).unwrap();
    m.restore_state(&snapshot).unwrap();
    assert!(!m.bus.dp.scaled_drawn(), "the shadow shows after a state read");
    assert_eq!(m.bus.dp.scale(), 2);
    assert!(m.bus.dp.multiple.rdram.iter().all(|&b| b == 0), "the shadow was not emptied");
    run(&mut m, &scene(0x4321_4321, false, 0, false));
    assert!(m.bus.dp.scaled_drawn());
    assert!(m.bus.dp.multiple.rdram[FRAMEBUFFER as usize * 4 + 1] != 0);
}

/// A change of the multiple between frames takes effect at the next list, empties the shadow, and leaves the machine as it was.
#[test]
fn the_multiple_changed_between_lists_takes_effect_at_the_next_and_leaves_the_machine_as_it_was() {
    let mut m = at(1, 4);
    run(&mut m, &scene(0x1111_1111, false, 0, false));
    let before = state(&m);
    m.set_scale(3);
    assert!(!m.bus.dp.scaled_drawn());
    assert!(m.bus.dp.threads.is_some(), "the drain was not restarted");
    run(&mut m, &scene(0x2222_2222, false, 0, false));
    assert!(m.bus.dp.scaled_drawn());
    let mut plain = at(1, 1);
    run(&mut plain, &scene(0x1111_1111, false, 0, false));
    assert!(state(&plain) == before);
    run(&mut plain, &scene(0x2222_2222, false, 0, false));
    assert!(state(&plain) == state(&m));
    m.set_scale(1);
    assert!(!m.bus.dp.scaled_drawn() && m.bus.dp.multiple.rdram.is_empty() && m.bus.dp.multiple.processor.is_none());
}

const VI_CONTROL: u32 = 0x0440_0000;

/// The VI over the fill scene, 320 by 240, sixteen bits, no resampling or filter, one source pixel an output pixel.
fn program_vi(m: &mut Machine) {
    let regs: [u32; 14] = {
        let mut r = [0u32; 14];
        r[0] = 2 | (3 << 8);
        r[1] = FRAMEBUFFER;
        r[2] = WIDTH;
        r[6] = 525;
        r[9] = (108 << 16) | (108 + WIDTH);
        r[10] = (34 << 16) | (34 + ROWS * 2);
        r[12] = 0x400;
        r[13] = 0x400;
        r
    };
    for (i, &word) in regs.iter().enumerate() {
        m.bus.write32(VI_CONTROL + i as u32 * 4, word);
    }
}

fn present(m: &mut Machine, out: &mut Scanout) -> (u32, u32, Vec<u8>) {
    out.repeat_rows = false;
    m.present_now(out);
    (out.width, out.height, out.frame.clone())
}

/// The scan-out at a multiple shows the multiple's raster N times as wide and as tall, and over a fill scene each of the console's pixels
/// becomes an N by N block of itself; averaged back down by N, it is the console's picture.
#[test]
fn the_scan_out_at_a_multiple_is_n_times_as_wide_and_the_fill_averages_back_to_the_console_s() {
    let mut one = at(1, 1);
    run(&mut one, &scene(0x1234_1234, false, 0, false));
    program_vi(&mut one);
    let mut out = Scanout::default();
    let (w1, h1, frame1) = present(&mut one, &mut out);
    assert_eq!((w1, h1), (640, 240));

    for scale in [2u32, 3, 4] {
        let mut scaled = at(scale as i32, 3);
        run(&mut scaled, &scene(0x1234_1234, false, 0, false));
        program_vi(&mut scaled);
        let mut out = Scanout::default();
        let (w, h, frame) = present(&mut scaled, &mut out);
        assert_eq!((w, h), (640 * scale, 240 * scale), "{scale}");
        let (w1, w) = (w1 as usize, w as usize);
        let n = scale as usize;
        for y in 0..240 {
            for x in 0..w1 {
                let want = &frame1[(y * w1 + x) * 4..(y * w1 + x) * 4 + 4];
                for j in 0..n {
                    for i in 0..n {
                        let at = ((y * n + j) * w + x * n + i) * 4;
                        assert!(&frame[at..at + 4] == want, "{scale}: pixel {x},{y} of the multiple {i},{j} differs");
                    }
                }
            }
        }

        out.average = scale as i32;
        scaled.present_now(&mut out);
        assert_eq!((out.width, out.height), (640, 240), "{scale}: averaged");
        assert!(out.frame == frame1, "{scale}: the average of the fill is not the fill at one");
    }
}

/// `The_average_of_a_square_is_its_rounded_mean_channel_by_channel`, C#'s case.
#[test]
fn the_average_of_a_square_is_its_rounded_mean_channel_by_channel() {
    #[rustfmt::skip]
    let source = [
        0u8, 10, 255, 255,  1, 20, 255, 255,  100, 0, 0, 255,  100, 0, 0, 255,
        1, 30, 255, 255,  1, 40, 255, 255,  100, 0, 0, 255,  104, 0, 0, 255,
    ];
    let mut into = [0u8; 8];
    box_average(&source, 4, 2, &mut into);
    assert_eq!(into, [1, 25, 255, 255, 101, 0, 0, 255]);
}

/// The deferred walk at a multiple captures the shadow's lines and shows, a frame late, the picture the immediate walk shows.
#[test]
fn a_deferred_walk_at_a_multiple_shows_the_immediate_picture_a_frame_late() {
    for scale in [2, 4] {
        let mut now = at(scale, 1);
        let mut later = at(scale, 4);
        later.set_deferred(true);
        let (mut out_now, mut out_later) = (Scanout::default(), Scanout::default());
        out_later.repeat_rows = false;
        let mut previous: Option<(u32, u32, Vec<u8>)> = None;
        for (i, color) in [0x1234_1234u32, 0x4321_4321, 0xFFFE_FFFE].into_iter().enumerate() {
            run(&mut now, &scene(color, false, 0, false));
            run(&mut later, &scene(color, false, 0, false));
            program_vi(&mut now);
            program_vi(&mut later);
            let shown_now = present(&mut now, &mut out_now);
            later.present(&mut out_later);
            let shown_later = (out_later.width, out_later.height, out_later.frame.clone());
            if let Some(previous) = previous {
                assert!(shown_later == previous, "{scale}: frame {i} deferred differs from the frame before at once");
            }
            previous = Some(shown_now);
        }
        assert!(later.join_presentation(&mut out_later));
        assert!((out_later.width, out_later.height, out_later.frame.clone()) == previous.unwrap());
    }
}

/// Through the `Core`, as the shim drives it: the multiple set between frames, the frame's shape following it, and the state untouched.
#[test]
fn the_core_s_multiple_is_set_between_frames_and_the_frame_follows_it() {
    let mut core = Core::new(at_once());
    core.scanout.repeat_rows = false;
    run(&mut core.machine, &scene(0x1234_1234, false, 0, false));
    program_vi(&mut core.machine);
    core.present();
    assert_eq!((core.scanout.width, core.scanout.height), (RASTER_WIDTH as u32, 240));
    let plain = core.machine.save_state_vec(false).unwrap();

    core.set_multiple(2, 1, false);
    core.present();
    assert_eq!((core.scanout.width, core.scanout.height), (640, 240), "nothing has drawn at the multiple yet, so the console's picture shows");
    run(&mut core.machine, &scene(0x1234_1234, false, 0, false));
    core.present();
    assert_eq!((core.scanout.width, core.scanout.height), (1280, 480));
    assert!(core.machine.save_state_vec(false).unwrap() == plain);

    core.set_multiple(2, 2, false);
    run(&mut core.machine, &scene(0x1234_1234, false, 0, false));
    core.present();
    assert_eq!((core.scanout.width, core.scanout.height), (1280, 480), "2x drawn at 4 and averaged by 2 shows at 2");
    core.set_multiple(1, 1, false);
    core.present();
    assert_eq!((core.scanout.width, core.scanout.height), (640, 240));
}

/// `A_frame_buffer_high_in_memory_scans_out_at_a_multiple_as_a_low_one_does`: an origin times the multiple squared passes what the register's
/// twenty-four bits name, and the walk aligns first and multiplies after (Mars_Video.md §2.11).
#[test]
fn a_frame_buffer_high_in_memory_scans_out_at_a_multiple_as_a_low_one_does() {
    const LOW: u32 = 0x0008_0000;
    const HIGH: u32 = 0x0030_0000;
    const AT: u32 = 0x0010_0000;
    for scale in [2, 3, 4] {
        for deferred in [false, true] {
            let picture = |framebuffer: u32| {
                let mut m = at(scale, 1);
                m.set_deferred(deferred);
                let mut list = vec![(0x2F << 56) | (3 << 52), (0x3F << 56) | (2 << 51) | (319 << 32) | framebuffer as u64, (0x2D << 56) | ((320u64 << 2) << 12) | (240 << 2)];
                for band in 0..8u64 {
                    list.push((0x37 << 56) | (0x0843_0843 * (band + 1)));
                    list.push((0x36 << 56) | ((319u64 << 2) << 44) | (((band * 30 + 29) << 2) << 32) | ((band * 30) << 2));
                }
                list.push(0x29 << 56);
                run_at(&mut m, &list, AT);
                assert!(m.bus.dp.scaled_drawn());
                program_vi(&mut m);
                m.bus.write32(VI_CONTROL + 4, framebuffer);
                let mut out = Scanout::default();
                out.repeat_rows = false;
                m.present(&mut out);
                m.join_presentation(&mut out);
                out.frame.clone()
            };
            let (low, high) = (picture(LOW), picture(HIGH));
            assert!(low.iter().any(|&b| b != 0));
            assert!(low == high, "at {scale}x {}: the high frame buffer's picture is not the low one's", if deferred { "deferred" } else { "immediate" });
        }
    }
}

fn run_at(m: &mut Machine, list: &[u64], at: u32) {
    hand_over(m, list, at);
    m.join_rdp();
}

/// A setting sent again unchanged, as the shim sent every setting before every frame Mistress ran, must leave a deferred walk out:
/// joining it threw away the overlap and cost 3 to 4 ms a frame on the handheld (Mars_Native.md §6.4.8).
#[test]
fn the_multiple_sent_again_unchanged_leaves_the_deferred_walk_out() {
    let mut machine = at_once();
    machine.set_deferred(true);
    let mut core = Core::new(machine);
    core.set_multiple(2, 1, false);
    for color in [0x1234_1234u32, 0x4321_4321] {
        run(&mut core.machine, &scene(color, false, 0, false));
        program_vi(&mut core.machine);
        core.present();
    }
    let joined = core.scanout.joined;
    core.set_multiple(2, 1, false);
    assert_eq!(core.scanout.joined, joined, "an unchanged multiple joined the walk out");
    core.set_multiple(1, 1, false);
    assert_eq!(core.scanout.joined, joined + 1, "a changed multiple must still join the walk made at the old one");
}
