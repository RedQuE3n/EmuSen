//! The chip's DR edges against an S-CPU that ignores RQM, for the transfers that are not handshaken
//! (VenusRT_Native.md §37.3, §42): `dsp_edges <chip> <command> <mode> <k> [inputs...]`. Mode `poll` answers k cycles
//! after each rise; `blind` writes the inputs one byte every k cycles from the command on and then reads every k
//! cycles. Each edge and access is printed with its chip cycle from the command byte; nothing goes to a file.
use venusrt::chips::dsporacle::*;
use venusrt::chips::necdsp::{Port, Transfer};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let stem = a[1].clone();
    let command = u8::from_str_radix(&a[2], 16).unwrap();
    let blind = a[3] == "blind";
    let k: u64 = a[4].parse().unwrap();
    let limit: u64 = std::env::var("EDGES_LIMIT").ok().and_then(|v| v.parse().ok()).unwrap_or(20_000);
    let inputs: Vec<u8> = a[5..].iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect();
    let image = firmware(&stem).unwrap_or_else(|| std::process::exit(1));
    let mut chip = idle_chip(&stem, &image, &mut Host::steady(), 0).unwrap();
    // EDGES_BEFORE, a command and its inputs in hex, runs first through the oracle's driver.
    if let Ok(before) = std::env::var("EDGES_BEFORE") {
        let w: Vec<u16> = before.split_whitespace().map(|v| u16::from_str_radix(v, 16).unwrap()).collect();
        transact(&mut chip, &mut Host::steady(), w[0] as u8, &w[1..]);
        for _ in 0..200 {
            chip.tick();
        }
    }
    chip.dsp.transfers.as_mut().unwrap().clear();
    let start = chip.dsp.cycles;
    chip.write(Port::Dr, command);
    let mut next = inputs.iter();
    let mut reads = 0;
    let mut since_rise: Option<u64> = None;
    let mut last_rqm = chip.status() & 0x80 != 0;
    let mut idle_edges = 0;
    let mut last_sr = chip.status();
    for n in 1..20_000u64 {
        chip.dsp.step();
        chip.dsp.cycles += 1;
        let rqm = chip.status() & 0x80 != 0;
        if chip.status() & 0x7F != last_sr & 0x7F {
            println!("{:6}     SR {:02X}", chip.dsp.cycles - start, chip.status());
        }
        last_sr = chip.status();
        if rqm && !last_rqm {
            since_rise = Some(0);
        }
        last_rqm = rqm;
        for (at, t) in chip.dsp.transfers.as_mut().unwrap().drain(..) {
            let c = at - start;
            match t {
                Transfer::ChipRead { .. } => println!("{c:6} chip read  DR={:04X}", chip.dsp.dr),
                Transfer::ChipWrite { value, .. } => {
                    println!("{c:6} chip write {value:04X}");
                    if value == 0xFF || value == 0x80 && stem != "dsp2" {
                        idle_edges += 1;
                    }
                }
                Transfer::HostWrite(v) => println!("{c:6}   host write {v:02X}"),
                Transfer::HostRead(v) => println!("{c:6}   host read  {v:02X}"),
                Transfer::HostRam(..) => {}
            }
        }
        let act = if blind { n % k == 0 } else { since_rise.is_some_and(|s| s == k) };
        if let Some(s) = since_rise.as_mut() {
            *s += 1;
        }
        if act {
            since_rise = None;
            let v = next.next().copied();
            for byte in 0..2 {
                if byte == 1 {
                    if chip.status() & 0x14 != 0x10 {
                        break;
                    }
                    for _ in 0..3 {
                        chip.dsp.step();
                        chip.dsp.cycles += 1;
                    }
                }
                match v {
                    Some(v) => chip.write(Port::Dr, v),
                    None => {
                        chip.read(Port::Dr);
                        reads += 1;
                    }
                }
            }
        }
        if reads > 300 || idle_edges > 1 || n > limit {
            break;
        }
    }
}
