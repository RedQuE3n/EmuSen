//! The chip-view, expression, trace and telemetry groups and the pictures and sound read in bulk: the C#
//! `SessionService.Views` but its cheats, over the debugger's views (EmuSen_DianaSh.md §9.2).

use emusen_crystal_debug::context::Set;
use emusen_crystal_debug::views;
use emusen_session_service::abi::{status, telemetry as kind};

use super::args::Raw;
use super::machine::dm;
use super::{Answer, Call, int32, not_supported, processor_of, refused, registers_json, space};
use crate::json::Json;

/// `[{"index","general","hdma","control","destination","destination_name","source","size","hdma_active","table",…}]`.
pub fn dma_channels(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let channels = m.map(|m| d.dma_channels(m)).unwrap_or_default();
        let mut j = Json::new();
        j.begin_array();
        for x in channels {
            j.begin_object().int("index", x.0 as i64).bool("general", x.1).bool("hdma", x.2).int("control", x.3 as i64).int("destination", x.4 as i64);
            j.str_or_null("destination_name", d.name_dma_destination(x.4).as_deref()).u64("source", x.5 as u32 as u64).int("size", x.6 as i64);
            j.bool("hdma_active", x.7).int("table", x.8 as i64).int("line_counter", x.9 as i64).int("indirect", x.10 as i64).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

/// `EMUSEN_SESSION_DMA_LOG_*`: no generic engine records its transfers.
pub fn dma_log_op(c: &Call, raw: Raw) -> Answer<Vec<u8>> {
    c.gated(|_, _| {
        raw.bytes()?;
        c.target()?;
        Err(not_supported("This core does not record DMA transfers."))
    })
}

/// `EMUSEN_SESSION_FLOW_*`: no generic engine watches a coprocessor's register window.
pub fn register_flow_op(c: &Call, raw: Raw) -> Answer<Vec<u8>> {
    c.gated(|_, _| {
        raw.bytes()?;
        c.target()?;
        Err(not_supported("This cartridge has no coprocessor register window to watch."))
    })
}

/// No generic engine keeps a coprocessor's register history.
pub fn coprocessor_history(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    Err(not_supported("This core does not keep coprocessor register history."))
}

/// `[{"name","address","width","mode"}]`: where each vector's pointer lives.
pub fn vectors(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let mut j = Json::new();
        j.begin_array();
        for v in e.debug.d.interrupt_vectors() {
            j.begin_object().str("name", v.0).u64("address", v.1 as u32 as u64).int("width", v.2 as i64).str("mode", v.3).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

/// `{"stride","text"}`: the pack's words for a tilemap entry.
pub fn tilemap_entry(c: &Call, name: Raw, address: u32) -> Answer<Vec<u8>> {
    let name = name.text()?;
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let named = space(d, m.as_deref(), name)?;
        let text = m.map(|m| d.decode_tilemap_entry(m, &named, address as i32)).unwrap_or_default();
        let mut j = Json::new();
        j.begin_object().int("stride", d.tilemap_entry_stride() as i64).str("text", &text).end_object();
        Ok(j.finish().into_bytes())
    })
}

/// A tile's 64 palette indices, none where the pack decodes no tiles; a depth it refuses is a fault in its words.
pub fn tile_pixels(c: &Call, name: Raw, address: u32, bpp: u32) -> Answer<Vec<u8>> {
    let name = name.text()?;
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let named = space(d, m.as_deref(), name)?;
        let bpp = int32(bpp as i64, "bpp")?;
        let Some(m) = m else { return Ok(Vec::new()) };
        d.decode_tile_pixels(m, &named, address as i32, bpp).map_err(|why| refused(status::FAULT, why))
    })
}

fn sprite(j: &mut Json, s: &views::Sprite) {
    j.begin_object().int("index", s.0 as i64).int("x", s.1 as i64).int("y", s.2 as i64).int("width", s.3 as i64).int("height", s.4 as i64);
    j.int("tile", s.5 as i64).int("palette", s.6 as i64).int("priority", s.7 as i64).bool("flip_x", s.8).bool("flip_y", s.9).end_object();
}

fn palette(j: &mut Json, p: &views::Palette) {
    j.begin_object().int("index", p.0 as i64).key_array("colors");
    for (r, g, b) in &p.1 {
        j.begin_array().int_value(*r as i64).int_value(*g as i64).int_value(*b as i64).end_array();
    }
    j.end_array().end_object();
}

fn channel(j: &mut Json, c: &views::AudioChannel) {
    j.begin_object().int("index", c.0 as i64).str("name", &c.1).bool("active", c.2).int("level", c.3 as i64).bool("muted", c.4).str("info", &c.5).end_object();
}

fn list<T>(items: &[T], write: fn(&mut Json, &T)) -> Vec<u8> {
    let mut j = Json::new();
    j.begin_array();
    for x in items {
        write(&mut j, x);
    }
    j.end_array();
    j.finish().into_bytes()
}

/// `{"max","sprites"}`, the sprites as the last refresh copied them.
pub fn sprites(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let all = m.map(|m| d.sprites(m)).unwrap_or_default();
        let mut j = Json::new();
        j.begin_object().int("max", d.max_sprites() as i64).key_array("sprites");
        for s in &all {
            sprite(&mut j, s);
        }
        j.end_array().end_object();
        Ok(j.finish().into_bytes())
    })
}

pub fn palettes(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        Ok(list(&m.map(|m| d.palettes(m)).unwrap_or_default(), palette))
    })
}

pub fn audio_channels(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        Ok(list(&m.map(|m| d.audio_channels(m)).unwrap_or_default(), channel))
    })
}

/// A channel muted or let through; the channel crosses as a 32-bit pattern, read as the core's signed index.
pub fn set_channel_muted(c: &Call, channel_index: u32, muted: bool) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        let (d, m) = dm(e);
        if let Some(m) = m {
            d.set_channel_muted(m, channel_index as i32, muted);
        }
        Ok(0)
    })
}

/// `{"value"}` of an expression in a processor's context; PARSE with the evaluator's words.
pub fn evaluate(c: &Call, processor: u32, text: Raw) -> Answer<Vec<u8>> {
    let text = text.text()?;
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        let p = processor_of(&e.debug.d, &core, processor)?;
        let (d, m) = dm(e);
        let Some(m) = m else { return Err(refused(status::NO_SESSION, "the session has no machine")) };
        let value = d.evaluate(m, p, text.unwrap_or("")).map_err(|why| refused(status::PARSE, why.message()))?;
        let mut j = Json::new();
        j.begin_object().int("value", value).end_object();
        Ok(j.finish().into_bytes())
    })
}

/// The symbols a processor's context names, those containing `filter` in any case where one is given.
pub fn symbols(c: &Call, processor: u32, filter: Raw) -> Answer<Vec<u8>> {
    let filter = filter.text()?;
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        let p = processor_of(&e.debug.d, &core, processor)?;
        let (d, m) = dm(e);
        let Some(m) = m else { return Err(refused(status::NO_SESSION, "the session has no machine")) };
        let names = d.symbol_names(m, p);
        let mut j = Json::new();
        j.begin_array();
        for n in names.iter().filter(|n| filter.is_none_or(|f| emusen_dotnet::text::contains_ignore_case(n, f))) {
            j.str_value(n);
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

/// `Trace(session, processor)`: no generic engine registers a trace switch.
fn trace_switch(c: &Call, processor: u32) -> Answer<()> {
    c.target()?;
    if processor != 0 {
        return Err(not_supported("Only the main processor can be traced."));
    }
    Err(not_supported("This core has no instruction trace."))
}

pub fn trace(c: &Call, processor: u32, count: u32) -> Answer<i64> {
    c.change(|_, _| {
        if count < 1 {
            return Err(refused(status::BAD_ARGUMENT, "Trace count must be at least 1."));
        }
        trace_switch(c, processor).map(|()| 0)
    })
}

pub fn trace_off(c: &Call, processor: u32) -> Answer<i64> {
    c.change(|_, _| trace_switch(c, processor).map(|()| 0))
}

/// `EMUSEN_SESSION_TELEMETRY_*` as JSON, each as the snapshots the last refresh took.
pub fn telemetry(c: &Call, which: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let registers = |set: Set| {
            let mut j = Json::new();
            registers_json(&mut j, &d.registers(set));
            j.finish().into_bytes()
        };
        Ok(match which {
            kind::CPU => registers(Set::Processor(0)),
            kind::VIDEO => registers(Set::Video),
            kind::APU => registers(Set::Apu),
            kind::COPROCESSOR => registers(Set::Coprocessor),
            kind::SPRITES => list(&m.map(|m| d.sprites(m)).unwrap_or_default(), sprite),
            kind::PALETTES => list(&m.map(|m| d.palettes(m)).unwrap_or_default(), palette),
            kind::AUDIO_CHANNELS => list(&m.map(|m| d.audio_channels(m)).unwrap_or_default(), channel),
            kind::HARDWARE_LOAD => {
                let mut j = Json::new();
                j.begin_array();
                for (name, percent) in d.load() {
                    j.begin_object().str("name", &name).f64("percent", percent).str("kind", "emulator_cost").end_object();
                }
                j.end_array();
                j.finish().into_bytes()
            }
            _ => return Err(not_supported(format!("no telemetry kind {which}"))),
        })
    })
}

/// A picture as the header lays it out: width and height as two little-endian uint32, then RGBA rows.
fn picture((rgba, width, height): views::Picture) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + rgba.len());
    bytes.extend_from_slice(&(width as u32).to_le_bytes());
    bytes.extend_from_slice(&(height as u32).to_le_bytes());
    bytes.extend_from_slice(&rgba);
    bytes
}

pub fn tile_sheet(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        Ok(picture(m.map(|m| d.tile_sheet(m)).unwrap_or_default()))
    })
}

pub fn palette_swatch(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        Ok(picture(m.map(|m| d.palette_swatch(m)).unwrap_or_default()))
    })
}

/// The rate the engine last reported as a little-endian uint32, then the samples held, not drained, as int16.
pub fn audio_peek(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let rate = e.audio_sample_rate();
        let (d, m) = dm(e);
        let samples = m.map(|m| d.audio_samples(m).0).unwrap_or_default();
        let mut bytes = Vec::with_capacity(4 + 2 * samples.len());
        bytes.extend_from_slice(&(rate as u32).to_le_bytes());
        for s in samples {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        Ok(bytes)
    })
}
