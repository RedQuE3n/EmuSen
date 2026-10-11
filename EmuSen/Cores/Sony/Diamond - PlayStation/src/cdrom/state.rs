//! The controller's part of the state (Diamond_CDROM.md §10): its registers, FIFOs, the command and responses in
//! flight, the drive, the sector buffer, the XA decoder and the audio ring, and the disc's identity; never the disc.

use emusen_native::{StateReader, StateWriter, Truncated};
use emusen_disc::SubQ;

use super::audio::RING;
use super::{Activity, Buffered, Cdrom, Pending, Reply, Then};

fn activity_code(a: Activity) -> u8 {
    match a {
        Activity::Stopped => 0,
        Activity::SpinningUp => 1,
        Activity::Idle => 2,
        Activity::Seeking => 3,
        Activity::Reading => 4,
        Activity::Playing => 5,
    }
}

fn activity_of(c: u8) -> Activity {
    match c {
        1 => Activity::SpinningUp,
        2 => Activity::Idle,
        3 => Activity::Seeking,
        4 => Activity::Reading,
        5 => Activity::Playing,
        _ => Activity::Stopped,
    }
}

fn then_code(t: Then) -> u8 {
    match t {
        Then::Hold { complete: false } => 0,
        Then::Hold { complete: true } => 1,
        Then::Read => 2,
        Then::Play => 3,
    }
}

fn then_of(c: u8) -> Then {
    match c {
        1 => Then::Hold { complete: true },
        2 => Then::Read,
        3 => Then::Play,
        _ => Then::Hold { complete: false },
    }
}

impl Cdrom {
    /// The disc's identity: its lead-out and each track's number, control and start, folded (FNV-1a).
    pub fn disc_id(&self) -> u64 {
        let Some(d) = &self.disc else { return 0 };
        let toc = d.get().toc();
        let mut h: u64 = 0xCBF2_9CE4_8422_2325;
        let mut eat = |v: u32| {
            for b in v.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x100_0000_01B3);
            }
        };
        eat(toc.lead_out);
        for (n, c, s) in toc.tracks {
            eat(n as u32 | (c as u32) << 8);
            eat(s);
        }
        h
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.u64("DiscId", self.disc_id());
        w.bytes("Registers", &[self.index, self.mask, self.flags, self.param_len, self.result_len, self.result_pos, self.hchpctl, self.mode, self.filter.0, self.filter.1, self.play_track]);
        w.bytes("Params", &self.params);
        w.bytes("Results", &self.results);
        w.u16s("DataAt", &[self.data_len, self.data_pos]);
        w.bytes("Data", &self.data[..]);
        w.bools("Flags", &[self.result_ready, self.command.is_some(), self.whole, self.setloc_pending, self.shell_flag, self.seek_error, self.id_error, self.crashed, self.motor, self.double, self.lid_open, self.pending_seek, self.seek_data, self.header_valid, self.waiting.is_some()]);
        w.bytes("Drive", &[self.command.unwrap_or(0), activity_code(self.activity), then_code(self.then)]);
        w.u32s("Positions", &[self.setloc, self.position, self.target, self.hover, self.resume]);
        w.u64s("Times", &[self.command_at, self.reset_until, self.drive_at, self.motor_stops_at, self.int1_at]);
        for (i, p) in self.queue.iter().enumerate() {
            w.group(["Queue0", "Queue1", "Queue2", "Queue3"][i], |w| {
                let (int, kind, bytes, len, at) = match p {
                    None => (0, 0, [0u8; 12], 0, 0),
                    Some(Pending { int, reply: Reply::Stat, at }) => (*int, 1, [0; 12], 0, *at),
                    Some(Pending { int, reply: Reply::Bytes(b, n), at }) => (*int, 2, *b, *n, *at),
                };
                w.bytes("Kind", &[kind, int, len]);
                w.bytes("Bytes", &bytes);
                w.u64("At", at);
            });
        }
        w.u32("DeliveredAt", self.delivered.at);
        w.bytes("Delivered", &self.delivered.raw[..]);
        let waiting = self.waiting.as_ref();
        w.u32("WaitingAt", waiting.map_or(0, |b| b.at));
        w.bool("WaitingSecondTry", waiting.is_some_and(|b| b.second_try));
        w.bytes("Waiting", waiting.map_or(&[0u8; 2352][..], |b| &b.raw[..]));
        w.bytes("Header", &self.header);
        w.bytes("Q", &self.q.0);
        let x = &self.xa;
        w.i32s("XaHistory", &[x.old[0][0], x.old[0][1], x.old[1][0], x.old[1][1]]);
        w.u16s("XaRing", &x.ring.as_flattened().iter().map(|&v| v as u16).collect::<Vec<_>>());
        w.bytes("XaStep", &[x.p, x.sixstep]);
        let o = &self.out;
        w.u16s("OutAt", &[o.head, o.len]);
        w.bytes("Volume", &[o.written[0], o.written[1], o.written[2], o.written[3], o.volume[0], o.volume[1], o.volume[2], o.volume[3]]);
        w.bools("OutFlags", &[o.xa_muted, o.muted, o.from_xa]);
        w.u16s("OutLeft", &o.left.map(|v| v as u16));
        w.u16s("OutRight", &o.right.map(|v| v as u16));
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let _disc = r.u64()?;
        let mut g = [0u8; 11];
        r.bytes(&mut g)?;
        [self.index, self.mask, self.flags, self.param_len, self.result_len, self.result_pos, self.hchpctl, self.mode] = [g[0] & 3, g[1], g[2], g[3].min(16), g[4].min(16), g[5] & 15, g[6], g[7]];
        self.filter = (g[8], g[9]);
        self.play_track = g[10];
        r.bytes(&mut self.params)?;
        r.bytes(&mut self.results)?;
        let mut d = [0u16; 2];
        r.u16s(&mut d)?;
        [self.data_len, self.data_pos] = [d[0].min(2352), d[1].min(2352)];
        r.bytes(&mut self.data[..])?;
        let mut f = [false; 15];
        r.bools(&mut f)?;
        let has_command = f[1];
        let waiting = f[14];
        [self.result_ready, self.whole, self.setloc_pending, self.shell_flag, self.seek_error, self.id_error, self.crashed, self.motor, self.double, self.lid_open, self.pending_seek, self.seek_data, self.header_valid] = [f[0], f[2], f[3], f[4], f[5], f[6], f[7], f[8], f[9], f[10], f[11], f[12], f[13]];
        let mut b = [0u8; 3];
        r.bytes(&mut b)?;
        self.command = has_command.then_some(b[0]);
        self.activity = activity_of(b[1]);
        self.then = then_of(b[2]);
        let mut p = [0u32; 5];
        r.u32s(&mut p)?;
        [self.setloc, self.position, self.target, self.hover, self.resume] = p;
        let mut t = [0u64; 5];
        r.u64s(&mut t)?;
        [self.command_at, self.reset_until, self.drive_at, self.motor_stops_at, self.int1_at] = t;
        for q in &mut self.queue {
            let mut k = [0u8; 3];
            r.bytes(&mut k)?;
            let mut bytes = [0u8; 12];
            r.bytes(&mut bytes)?;
            let at = r.u64()?;
            *q = match k[0] {
                1 => Some(Pending { int: k[1], reply: Reply::Stat, at }),
                2 => Some(Pending { int: k[1], reply: Reply::Bytes(bytes, k[2].min(12)), at }),
                _ => None,
            };
        }
        self.delivered.at = r.u32()?;
        r.bytes(&mut self.delivered.raw[..])?;
        let at = r.u32()?;
        let second_try = r.bool()?;
        let mut raw = Box::new([0u8; 2352]);
        r.bytes(&mut raw[..])?;
        self.waiting = waiting.then_some(Buffered { raw, at, second_try });
        r.bytes(&mut self.header)?;
        let mut q = [0u8; 12];
        r.bytes(&mut q)?;
        self.q = SubQ(q);
        let mut h = [0i32; 4];
        r.i32s(&mut h)?;
        self.xa.old = [[h[0], h[1]], [h[2], h[3]]];
        let mut ring = [0u16; 64];
        r.u16s(&mut ring)?;
        for (i, v) in ring.iter().enumerate() {
            self.xa.ring[i / 32][i % 32] = *v as i16;
        }
        let mut s = [0u8; 2];
        r.bytes(&mut s)?;
        self.xa.p = s[0];
        self.xa.sixstep = s[1].clamp(1, 6);
        let mut at = [0u16; 2];
        r.u16s(&mut at)?;
        self.out.head = at[0] % RING as u16;
        self.out.len = at[1].min(RING as u16);
        let mut v = [0u8; 8];
        r.bytes(&mut v)?;
        self.out.written.copy_from_slice(&v[..4]);
        self.out.volume.copy_from_slice(&v[4..]);
        let mut of = [false; 3];
        r.bools(&mut of)?;
        [self.out.xa_muted, self.out.muted, self.out.from_xa] = of;
        let mut ch = vec![0u16; RING];
        r.u16s(&mut ch)?;
        for (d, s) in self.out.left.iter_mut().zip(&ch) {
            *d = *s as i16;
        }
        r.u16s(&mut ch)?;
        for (d, s) in self.out.right.iter_mut().zip(&ch) {
            *d = *s as i16;
        }
        self.refresh();
        Ok(())
    }
}
