//! The script over plain data: registers, labels, spaces and units a script sets, the contexts and the registries over
//! them, with no machine. `CrystalDebugScript.Fake` in WiseMan runs the same verbs over the C# classes.

use emusen_crystal_debug::breakpoints::Evaluate;
use emusen_crystal_debug::conditions;
use emusen_crystal_debug::context::{CpuContext, Extra, Live, Register, Set, Source, TargetContext, UnitInfo, UnitSymbols};
use emusen_crystal_debug::expr::{self, Context, Evaluator};
use emusen_crystal_debug::labels::LabelRegistry;
use emusen_dotnet::text::equals_ignore_case;

fn set_named(word: &str) -> Option<Set> {
    Some(match word {
        "cpu" => Set::Processor(0),
        "video" => Set::Video,
        "apu" => Set::Apu,
        "cop" => Set::Coprocessor,
        _ => return None,
    })
}

const SETS: [Set; 4] = [Set::Processor(0), Set::Video, Set::Apu, Set::Coprocessor];

fn index(set: Set) -> usize {
    SETS.iter().position(|&s| s == set).unwrap_or(usize::MAX)
}

/// What a fake target holds.
#[derive(Default)]
pub struct Data {
    live: [Vec<Register>; 4],
    shown: [Vec<Register>; 4],
    pub labels: LabelRegistry,
    spaces: Vec<(String, Vec<u8>)>,
    frame: i64,
    stack: Option<i64>,
    pack: Vec<(String, i64)>,
    refreshes: usize,
}

impl Source for Data {
    fn frame(&mut self) -> i64 {
        self.frame
    }

    fn label(&self, name: &str) -> Option<i32> {
        self.labels.address_of(name)
    }

    fn label_names(&self) -> Vec<String> {
        self.labels.all().map(|(_, n, _)| n.to_string()).collect()
    }

    fn space_count(&self) -> usize {
        self.spaces.len()
    }

    fn space_name(&self, index: usize) -> &str {
        &self.spaces[index].0
    }

    fn space_size(&mut self, index: usize) -> i32 {
        self.spaces[index].1.len() as i32
    }

    fn space_read(&mut self, index: usize, address: i32) -> u8 {
        self.spaces[index].1[address as usize]
    }

    fn registers(&mut self, set: Set) -> Vec<Register> {
        self.shown.get(index(set)).cloned().unwrap_or_default()
    }

    fn register(&mut self, set: Set, name: &str) -> Option<u64> {
        self.shown.get(index(set))?.iter().find(|r| equals_ignore_case(&r.name, name)).map(|r| r.value)
    }

    fn unit_registers(&mut self, _: u32, _: &mut [i64]) -> usize {
        0
    }

    fn pack_symbol(&mut self, name: &str) -> Option<i64> {
        self.pack.iter().find(|(n, _)| equals_ignore_case(n, name)).map(|&(_, v)| v)
    }

    fn stack_depth(&self) -> Option<i64> {
        self.stack
    }

    fn refresh_registers(&mut self) {
        // DebugExpressions.RefreshRegisters on a target with no processors: the main set, the video and the sound rows.
        self.refreshes += 1;
        for i in 0..3 {
            self.shown[i] = self.live[i].clone();
        }
    }
}

#[derive(Clone)]
enum Chosen {
    Target,
    Cpu(Set, Option<String>),
    Main,
}

pub struct Script {
    pub data: Data,
    pub registries: crate::regs::Registries,
    chosen: Chosen,
    units: Vec<UnitInfo>,
    taken: Vec<String>,
}

fn number(word: &str) -> i64 {
    let w = word.trim();
    if let Some(hex) = w.strip_prefix("0x").or_else(|| w.strip_prefix('$')) {
        return u64::from_str_radix(hex, 16).map(|v| v as i64).unwrap_or(0);
    }
    w.parse().unwrap_or(0)
}

impl Script {
    pub fn new() -> Script {
        Script { data: Data::default(), registries: crate::regs::Registries::new(), chosen: Chosen::Target, units: Vec::new(), taken: Vec::new() }
    }

    fn pack_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.data.pack.iter().map(|(n, _)| n.clone()).collect();
        if self.data.stack.is_some() {
            names.push("stackdepth".to_string());
        }
        names
    }

    /// The chosen context over the data, wrapped as a breakpoint's live scope when `live`.
    fn with_context<T>(&mut self, live: bool, f: impl FnOnce(&mut dyn Context) -> T) -> T {
        let names = self.pack_names();
        context(&mut self.data, &self.chosen, &names, live, f)
    }

    /// The evaluator over the chosen context, beside the registries it serves.
    pub fn split(&mut self) -> (FakeEval<'_>, &mut crate::regs::Registries) {
        let names = self.pack_names();
        (FakeEval { data: &mut self.data, chosen: &self.chosen, names }, &mut self.registries)
    }

    pub fn run(&mut self, verb: &str, rest: &str) -> String {
        let words: Vec<&str> = rest.split(' ').filter(|w| !w.is_empty()).collect();
        match verb {
            "regs" => {
                let Some(set) = words.first().and_then(|w| set_named(w)) else { return "bad set".into() };
                let rows: Vec<Register> = words[1..]
                    .iter()
                    .map(|w| {
                        let (name, v) = w.split_once('=').unwrap_or((w, "0"));
                        let (value, bits) = v.split_once('/').unwrap_or((v, "8"));
                        Register { name: name.to_string(), value: number(value) as u64, bits: number(bits) as i32 }
                    })
                    .collect();
                self.data.live[index(set)] = rows.clone();
                self.data.shown[index(set)] = rows;
                "ok".into()
            }
            "set" => {
                let Some(set) = words.first().and_then(|w| set_named(w)) else { return "bad set".into() };
                if let Some(r) = self.data.live[index(set)].iter_mut().find(|r| r.name == words[1]) {
                    r.value = number(words[2]) as u64;
                }
                "ok".into()
            }
            "refresh" => {
                self.data.refresh_registers();
                "ok".into()
            }
            "refreshes" => self.data.refreshes.to_string(),
            "label" => {
                let comment = if words.len() > 2 { Some(words[2..].join(" ")) } else { None };
                self.data.labels.add(number(words[0]) as i32, words[1], comment.as_deref());
                "ok".into()
            }
            "unlabel" => self.data.labels.remove_by_name(rest).to_string(),
            "unlabelat" => self.data.labels.remove_by_address(number(rest) as i32).to_string(),
            "labels" => self.data.labels.all().map(|(a, n, c)| format!("{a}:{n}:{}", c.unwrap_or("-"))).collect::<Vec<_>>().join(","),
            "labelload" => {
                let text = rest.replace("\\n", "\n").replace("\\t", "\t");
                let (added, errors) = self.data.labels.load_from_lines(text.split('\n'));
                format!("{added} [{}]", errors.join("|"))
            }
            "labelsave" => self.data.labels.to_lines().join("|"),
            "describe" => self.data.labels.describe(number(words[0]) as i32, number(words[1]) as usize),
            "space" => {
                let size = number(words[1]) as usize;
                let mut bytes = vec![0u8; size];
                for (i, b) in words[2..].iter().enumerate() {
                    if i < size {
                        bytes[i] = u8::from_str_radix(b, 16).unwrap_or(0);
                    }
                }
                self.data.spaces.push((words[0].to_string(), bytes));
                "ok".into()
            }
            "frame" => {
                self.data.frame = number(rest);
                "ok".into()
            }
            "stack" => {
                self.data.stack = if rest == "none" { None } else { Some(number(rest)) };
                "ok".into()
            }
            "pack" => {
                self.data.pack.push((words[0].to_string(), number(words[1])));
                "ok".into()
            }
            "context" => {
                self.chosen = match words.first().copied() {
                    Some("cpu") => Chosen::Cpu(words.get(1).and_then(|w| set_named(w)).unwrap_or(Set::Processor(0)), words.get(2).map(|s| s.to_string())),
                    Some("main") => Chosen::Main,
                    _ => Chosen::Target,
                };
                "ok".into()
            }
            "eval" => self.with_context(false, |c| match Evaluator::new(c).evaluate(rest) {
                Ok(v) => expr::format(v),
                Err(e) => format!("! {}", e.message()),
            }),
            "check" => self.with_context(false, |c| match Evaluator::new(c).check(rest) {
                Ok(()) => "ok".to_string(),
                Err(e) => format!("! {}", e.message()),
            }),
            "cond" => self.with_context(true, |c| match conditions::condition(c, rest) {
                Ok(v) => v.to_string(),
                Err(e) => format!("! {e}"),
            }),
            "render" => self.with_context(true, |c| match conditions::render(c, rest) {
                Ok(t) => t,
                Err(e) => format!("! {e}"),
            }),
            "prepare" => {
                let (c, l) = rest.split_once(" | ").unwrap_or((rest, "-"));
                let c = (c != "-").then_some(c);
                let l = (l != "-").then_some(l);
                self.with_context(false, |ctx| match conditions::prepare(ctx, c, l) {
                    Ok(any) => any.to_string(),
                    Err(e) => format!("! {e}"),
                })
            }
            "symbols" => self.with_context(false, |c| c.symbol_names().join(",")),
            "format" => expr::format(number(rest)),
            "unit" => {
                // unit ID NAME KIND REG/BITS...
                let registers = words[3..].iter().map(|w| {
                    let (n, b) = w.split_once('/').unwrap_or((w, "8"));
                    (n.to_string(), number(b) as i32)
                });
                self.units.push(UnitInfo { id: number(words[0]) as u32, name: words[1].replace('~', " "), kind: words[2].to_string(), registers: registers.collect() });
                "ok".into()
            }
            "taken" => {
                self.taken = words.iter().map(|w| w.to_string()).collect();
                "ok".into()
            }
            "unitnames" => UnitSymbols::new(&self.units, &self.taken).names.join(","),
            "unitfind" => match UnitSymbols::new(&self.units, &self.taken).find(rest) {
                Some((u, i, n)) => format!("{u} {i} {n}"),
                None => "none".into(),
            },
            _ => crate::regs::run(self, verb, rest, &words).unwrap_or_else(|| format!("unknown verb {verb}")),
        }
    }
}

/// The chosen context over `data`, wrapped as a breakpoint's live scope when `live`.
fn context<T>(data: &mut Data, chosen: &Chosen, names: &[String], live: bool, f: impl FnOnce(&mut dyn Context) -> T) -> T {
    let source: &mut dyn Source = data;
    match chosen {
        Chosen::Target => {
            let scope = TargetContext { source };
            if live { f(&mut Live { scope, fresh: false }) } else { f(&mut { scope }) }
        }
        Chosen::Cpu(set, space) => {
            let scope = CpuContext { source, set: *set, default_space: space.as_deref(), extra: Extra { names: &[], main: false } };
            if live { f(&mut Live { scope, fresh: false }) } else { f(&mut { scope }) }
        }
        Chosen::Main => {
            let scope = CpuContext { source, set: Set::Processor(0), default_space: None, extra: Extra { names, main: true } };
            if live { f(&mut Live { scope, fresh: false }) } else { f(&mut { scope }) }
        }
    }
}

/// The registry's evaluator over the chosen context, as BreakpointExpressions points one.
pub struct FakeEval<'a> {
    pub data: &'a mut Data,
    chosen: &'a Chosen,
    names: Vec<String>,
}

impl FakeEval<'_> {
    /// The chosen context, unwrapped, for checking an expression before it is added.
    pub fn scope<T>(&mut self, f: impl FnOnce(&mut dyn Context) -> T) -> T {
        context(self.data, self.chosen, &self.names, false, f)
    }
}

impl Evaluate for FakeEval<'_> {
    fn condition(&mut self, condition: &str) -> Result<bool, String> {
        context(self.data, self.chosen, &self.names, true, |c| conditions::condition(c, condition))
    }

    fn render(&mut self, log: &str) -> Result<String, String> {
        context(self.data, self.chosen, &self.names, true, |c| conditions::render(c, log))
    }
}

impl Data {
    /// A byte of a space by its name, 0 where there is none.
    pub fn byte(&self, space: &str, address: i32) -> u8 {
        if address < 0 {
            return 0;
        }
        self.spaces.iter().find(|(n, _)| equals_ignore_case(n, space)).and_then(|(_, b)| b.get(address as usize).copied()).unwrap_or(0)
    }

    pub fn put(&mut self, space: &str, address: i32, value: u8) {
        if let Some((_, b)) = self.spaces.iter_mut().find(|(n, _)| equals_ignore_case(n, space)) {
            if address >= 0 && (address as usize) < b.len() {
                b[address as usize] = value;
            }
        }
    }

    pub fn frame_now(&self) -> i64 {
        self.frame
    }
}
