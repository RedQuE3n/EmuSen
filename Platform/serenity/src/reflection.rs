//! What a stage binds, read from its SPIR-V's names and decorations: the C# `SpirvReflection`. See EmuSen_Serenity.md
//! §7.3, and §10.3 for the inputs left out of an entry point.
//!
//! A module that is not well formed fails here as it fails in C#, by the same exception: the words of the three the
//! C# throws itself, and the kind of the two its array accesses throw.

use std::collections::{HashMap, HashSet};

/// A member of the uniform block or the push constants: where it sits and how many bytes it takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

/// A uniform block or the push-constant block; `binding` is meaningless for the latter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub binding: u32,
    pub size: u32,
    pub members: Vec<Member>,
}

impl Block {
    pub fn find(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|member| member.name == name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sampler {
    pub name: String,
    pub binding: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reflection {
    pub uniforms: Option<Block>,
    pub push_constants: Option<Block>,
    pub samplers: Vec<Sampler>,
}

/// Why a module could not be read, as the C# exception it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// `ArgumentException`, with its words.
    NotSpirv(String),
    /// `IndexOutOfRangeException`: an instruction shorter than its kind needs.
    IndexOutOfRange,
    /// `ArgumentOutOfRangeException`: an entry point whose name runs to its end.
    ArgumentOutOfRange,
    /// Types that contain themselves, for which the C# does not return.
    Endless,
}

const MAGIC: u32 = 0x0723_0203;
const OP_NAME: u32 = 5;
const OP_MEMBER_NAME: u32 = 6;
const OP_ENTRY_POINT: u32 = 15;
const OP_TYPE_INT: u32 = 21;
const OP_TYPE_FLOAT: u32 = 22;
const OP_TYPE_VECTOR: u32 = 23;
const OP_TYPE_MATRIX: u32 = 24;
const OP_TYPE_IMAGE: u32 = 25;
const OP_TYPE_SAMPLED_IMAGE: u32 = 27;
const OP_TYPE_ARRAY: u32 = 28;
const OP_TYPE_STRUCT: u32 = 30;
const OP_TYPE_POINTER: u32 = 32;
const OP_CONSTANT: u32 = 43;
const OP_FUNCTION: u32 = 54;
const OP_VARIABLE: u32 = 59;
const OP_DECORATE: u32 = 71;
const OP_MEMBER_DECORATE: u32 = 72;
const DECORATION_BINDING: u32 = 33;
const DECORATION_OFFSET: u32 = 35;
const DECORATION_ARRAY_STRIDE: u32 = 6;
const DECORATION_MATRIX_STRIDE: u32 = 7;
const STORAGE_UNIFORM_CONSTANT: u32 = 0;
const STORAGE_INPUT: u32 = 1;
const STORAGE_UNIFORM: u32 = 2;
const STORAGE_PUSH_CONSTANT: u32 = 9;

/// How deep one type may hold another before the module is taken to hold itself.
const MAX_TYPE_DEPTH: usize = 256;

fn words_of(spirv: &[u8]) -> Vec<u32> {
    spirv.as_chunks::<4>().0.iter().map(|word| u32::from_le_bytes(*word)).collect()
}

/// An operand a kind of instruction needs: the C#'s array access, which throws when it is not there.
fn at(operands: &[u32], index: usize) -> Result<u32, Fault> {
    operands.get(index).copied().ok_or(Fault::IndexOutOfRange)
}

/// A SPIR-V literal string: UTF-8, zero-terminated, packed four bytes to a word.
fn text(operands: &[u32], from: usize) -> String {
    let bytes: Vec<u8> = operands.iter().skip(from).flat_map(|word| word.to_le_bytes()).take_while(|&b| b != 0).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

struct Module {
    names: HashMap<u32, String>,
    member_names: HashMap<(u32, u32), String>,
    bindings: HashMap<u32, u32>,
    member_offsets: HashMap<(u32, u32), u32>,
    array_strides: HashMap<u32, u32>,
    matrix_strides: HashMap<(u32, u32), u32>,
    types: HashMap<u32, (u32, Vec<u32>)>,
    constants: HashMap<u32, u32>,
}

impl Module {
    fn size_of(&self, kind: u32, matrix_stride: u32, depth: usize) -> Result<u32, Fault> {
        if depth > MAX_TYPE_DEPTH {
            return Err(Fault::Endless);
        }
        let Some((op, operands)) = self.types.get(&kind) else { return Ok(0) };
        Ok(match *op {
            OP_TYPE_INT | OP_TYPE_FLOAT => at(operands, 0)? / 8,
            OP_TYPE_VECTOR => self.size_of(at(operands, 0)?, 0, depth + 1)?.wrapping_mul(at(operands, 1)?),
            OP_TYPE_MATRIX => {
                let columns = at(operands, 1)?;
                columns.wrapping_mul(if matrix_stride != 0 { matrix_stride } else { self.size_of(at(operands, 0)?, 0, depth + 1)? })
            }
            OP_TYPE_ARRAY => {
                let element = match self.array_strides.get(&kind) {
                    Some(stride) => *stride,
                    None => self.size_of(at(operands, 0)?, matrix_stride, depth + 1)?,
                };
                element.wrapping_mul(self.constants.get(&at(operands, 1)?).copied().unwrap_or(1))
            }
            OP_TYPE_STRUCT => self.struct_size(kind, operands, depth + 1)?,
            _ => 0,
        })
    }

    fn struct_size(&self, kind: u32, members: &[u32], depth: usize) -> Result<u32, Fault> {
        let mut end = 0u32;
        for (m, member) in members.iter().enumerate() {
            let m = m as u32;
            let offset = self.member_offsets.get(&(kind, m)).copied().unwrap_or(0);
            let size = self.size_of(*member, self.matrix_strides.get(&(kind, m)).copied().unwrap_or(0), depth)?;
            end = end.max(offset.wrapping_add(size));
        }
        Ok(end)
    }

    fn block(&self, variable: u32, kind: u32) -> Result<Block, Fault> {
        let members = &self.types[&kind].1;
        let mut list = Vec::new();
        for (m, member) in members.iter().enumerate() {
            let m = m as u32;
            let name = self.member_names.get(&(kind, m)).cloned().unwrap_or_else(|| format!("_m{m}"));
            let offset = self.member_offsets.get(&(kind, m)).copied().unwrap_or(0);
            list.push(Member { name, offset, size: self.size_of(*member, self.matrix_strides.get(&(kind, m)).copied().unwrap_or(0), 0)? });
        }
        Ok(Block { binding: self.bindings.get(&variable).copied().unwrap_or(0), size: self.struct_size(kind, members, 0)?, members: list })
    }
}

pub fn read(spirv: &[u8]) -> Result<Reflection, Fault> {
    if spirv.len() < 20 || !spirv.len().is_multiple_of(4) {
        return Err(Fault::NotSpirv("Not SPIR-V: too short or not whole words.".to_string()));
    }
    let words = words_of(spirv);
    if words[0] != MAGIC {
        return Err(Fault::NotSpirv("Not SPIR-V: the magic number is wrong.".to_string()));
    }

    let mut module = Module {
        names: HashMap::new(),
        member_names: HashMap::new(),
        bindings: HashMap::new(),
        member_offsets: HashMap::new(),
        array_strides: HashMap::new(),
        matrix_strides: HashMap::new(),
        types: HashMap::new(),
        constants: HashMap::new(),
    };
    let mut variables: Vec<(u32, u32, u32)> = Vec::new();

    let mut index = 5;
    while index < words.len() {
        let (count, op) = ((words[index] >> 16) as usize, words[index] & 0xFFFF);
        if count == 0 || index + count > words.len() {
            return Err(Fault::NotSpirv(format!("Not SPIR-V: an instruction at word {index} runs past the end.")));
        }
        let operands = &words[index + 1..index + count];
        match op {
            OP_NAME => {
                module.names.insert(at(operands, 0)?, text(operands, 1));
            }
            OP_MEMBER_NAME => {
                module.member_names.insert((at(operands, 0)?, at(operands, 1)?), text(operands, 2));
            }
            OP_DECORATE => match at(operands, 1)? {
                DECORATION_BINDING => {
                    module.bindings.insert(at(operands, 0)?, at(operands, 2)?);
                }
                DECORATION_ARRAY_STRIDE => {
                    module.array_strides.insert(at(operands, 0)?, at(operands, 2)?);
                }
                _ => {}
            },
            OP_MEMBER_DECORATE => match at(operands, 2)? {
                DECORATION_OFFSET => {
                    module.member_offsets.insert((at(operands, 0)?, at(operands, 1)?), at(operands, 3)?);
                }
                DECORATION_MATRIX_STRIDE => {
                    module.matrix_strides.insert((at(operands, 0)?, at(operands, 1)?), at(operands, 3)?);
                }
                _ => {}
            },
            OP_TYPE_INT | OP_TYPE_FLOAT | OP_TYPE_VECTOR | OP_TYPE_MATRIX | OP_TYPE_IMAGE | OP_TYPE_SAMPLED_IMAGE | OP_TYPE_ARRAY | OP_TYPE_STRUCT | OP_TYPE_POINTER => {
                module.types.insert(at(operands, 0)?, (op, operands[1..].to_vec()));
            }
            OP_CONSTANT if operands.len() >= 3 => {
                module.constants.insert(operands[1], operands[2]);
            }
            OP_VARIABLE => variables.push((at(operands, 0)?, at(operands, 1)?, at(operands, 2)?)),
            _ => {}
        }
        index += count;
    }

    let mut reflection = Reflection::default();
    for (pointer, id, storage) in variables {
        let pointee = match module.types.get(&pointer) {
            Some((OP_TYPE_POINTER, operands)) => at(operands, 1)?,
            _ => pointer,
        };
        let Some((op, _)) = module.types.get(&pointee) else { continue };
        if storage == STORAGE_UNIFORM && *op == OP_TYPE_STRUCT {
            reflection.uniforms = Some(module.block(id, pointee)?);
        } else if storage == STORAGE_PUSH_CONSTANT && *op == OP_TYPE_STRUCT {
            reflection.push_constants = Some(module.block(id, pointee)?);
        } else if storage == STORAGE_UNIFORM_CONSTANT
            && *op == OP_TYPE_SAMPLED_IMAGE
            && let Some(name) = module.names.get(&id)
        {
            reflection.samplers.push(Sampler { name: name.clone(), binding: module.bindings.get(&id).copied().unwrap_or(0) });
        }
    }
    Ok(reflection)
}

/// The module with every Input variable no instruction reads left out of its entry points' interfaces, or none when
/// there is nothing to leave out and the caller keeps what it has.
pub fn without_unread_inputs(spirv: &[u8]) -> Result<Option<Vec<u8>>, Fault> {
    let words = words_of(spirv);
    if words.len() < 5 || words[0] != MAGIC {
        return Ok(None);
    }

    // A variable is read only inside a function; any word there equal to its id counts, so a literal that happens to match keeps an input, never drops one.
    let (mut inputs, mut read) = (HashSet::new(), HashSet::new());
    let mut entries: Vec<(usize, usize)> = Vec::new();
    let mut in_functions = false;
    let mut index = 5;
    while index < words.len() {
        let (count, op) = ((words[index] >> 16) as usize, words[index] & 0xFFFF);
        if count == 0 || index + count > words.len() {
            return Ok(None);
        }
        in_functions |= op == OP_FUNCTION;
        if in_functions {
            read.extend(words[index + 1..index + count].iter().copied());
        } else if op == OP_VARIABLE && count >= 4 && words[index + 3] == STORAGE_INPUT {
            inputs.insert(words[index + 2]);
        } else if op == OP_ENTRY_POINT {
            let mut name = index + 3;
            while name < index + count && words[name].to_le_bytes().iter().all(|&b| b != 0) {
                name += 1;
            }
            entries.push((index, name + 1));
        }
        index += count;
    }

    if !inputs.iter().any(|id| !read.contains(id)) {
        return Ok(None);
    }
    let unread = |id: &u32| inputs.contains(id) && !read.contains(id);
    let mut kept: Vec<u32> = Vec::with_capacity(words.len());
    let mut copied = 0;
    for (start, first) in entries {
        let end = start + (words[start] >> 16) as usize;
        kept.extend_from_slice(&words[copied..start]);
        // A name that runs to the instruction's end leaves no interface, and the C#'s range over it throws.
        if first > end {
            return Err(Fault::ArgumentOutOfRange);
        }
        let interface: Vec<u32> = words[first..end].iter().copied().filter(|id| !unread(id)).collect();
        kept.push(((first - start + interface.len()) as u32) << 16 | OP_ENTRY_POINT);
        kept.extend_from_slice(&words[start + 1..first]);
        kept.extend(interface);
        copied = end;
    }
    kept.extend_from_slice(&words[copied..]);
    Ok(Some(kept.iter().flat_map(|word| word.to_le_bytes()).collect()))
}

/// The two stages as one pass: their blocks' members together, and every sampler either names.
pub fn merge(vertex: &Reflection, fragment: &Reflection) -> Reflection {
    fn join(a: &Option<Block>, b: &Option<Block>) -> Option<Block> {
        let (Some(a), Some(b)) = (a, b) else { return a.clone().or_else(|| b.clone()) };
        let mut members = a.members.clone();
        members.extend(b.members.iter().filter(|member| a.find(&member.name).is_none()).cloned());
        Some(Block { binding: a.binding, size: a.size.max(b.size), members })
    }
    let mut samplers: Vec<Sampler> = Vec::new();
    for sampler in vertex.samplers.iter().chain(&fragment.samplers) {
        if !samplers.iter().any(|known| known.name == sampler.name) {
            samplers.push(sampler.clone());
        }
    }
    Reflection { uniforms: join(&vertex.uniforms, &fragment.uniforms), push_constants: join(&vertex.push_constants, &fragment.push_constants), samplers }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(instructions: &[&[u32]]) -> Vec<u8> {
        let mut words = vec![MAGIC, 0x0001_0000, 0, 100, 0];
        for instruction in instructions {
            words.push(((instruction.len() as u32) << 16) | instruction[0]);
            words.extend_from_slice(&instruction[1..]);
        }
        words.iter().flat_map(|word| word.to_le_bytes()).collect()
    }

    fn name(text: &str) -> Vec<u32> {
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        bytes.resize(bytes.len().div_ceil(4) * 4, 0);
        bytes.as_chunks::<4>().0.iter().map(|word| u32::from_le_bytes(*word)).collect()
    }

    #[test]
    fn a_block_its_members_and_a_sampler_are_read_from_names_and_decorations() {
        let mut member = vec![OP_MEMBER_NAME, 10, 1];
        member.extend(name("OutputSize"));
        let mut sampler = vec![OP_NAME, 31];
        sampler.extend(name("Source"));
        let spirv = module(&[
            &member,
            &sampler,
            &[OP_DECORATE, 20, DECORATION_BINDING, 3],
            &[OP_DECORATE, 31, DECORATION_BINDING, 2],
            &[OP_MEMBER_DECORATE, 10, 0, DECORATION_OFFSET, 0],
            &[OP_MEMBER_DECORATE, 10, 0, DECORATION_MATRIX_STRIDE, 16],
            &[OP_MEMBER_DECORATE, 10, 1, DECORATION_OFFSET, 64],
            &[OP_TYPE_FLOAT, 1, 32],
            &[OP_TYPE_VECTOR, 2, 1, 4],
            &[OP_TYPE_MATRIX, 3, 2, 4],
            &[OP_TYPE_STRUCT, 10, 3, 2],
            &[OP_TYPE_POINTER, 11, STORAGE_UNIFORM, 10],
            &[OP_VARIABLE, 11, 20, STORAGE_UNIFORM],
            &[OP_TYPE_IMAGE, 28, 1],
            &[OP_TYPE_SAMPLED_IMAGE, 29, 28],
            &[OP_TYPE_POINTER, 30, STORAGE_UNIFORM_CONSTANT, 29],
            &[OP_VARIABLE, 30, 31, STORAGE_UNIFORM_CONSTANT],
        ]);
        let reflection = read(&spirv).unwrap();
        let block = reflection.uniforms.unwrap();
        assert_eq!((block.binding, block.size), (3, 80));
        assert_eq!(block.members, [Member { name: "_m0".into(), offset: 0, size: 64 }, Member { name: "OutputSize".into(), offset: 64, size: 16 }]);
        assert_eq!(reflection.samplers, [Sampler { name: "Source".into(), binding: 2 }]);
        assert!(reflection.push_constants.is_none());
    }

    #[test]
    fn what_is_not_spirv_fails_as_the_csharp_fails() {
        assert_eq!(read(&[0; 16]), Err(Fault::NotSpirv("Not SPIR-V: too short or not whole words.".into())));
        assert_eq!(read(&[0; 21]), Err(Fault::NotSpirv("Not SPIR-V: too short or not whole words.".into())));
        assert_eq!(read(&[0; 20]), Err(Fault::NotSpirv("Not SPIR-V: the magic number is wrong.".into())));
        let mut cut = module(&[&[OP_TYPE_FLOAT, 1, 32]]);
        cut.truncate(cut.len() - 4);
        assert_eq!(read(&cut), Err(Fault::NotSpirv("Not SPIR-V: an instruction at word 5 runs past the end.".into())));
        assert_eq!(read(&module(&[&[OP_NAME]])), Err(Fault::IndexOutOfRange));
        assert_eq!(read(&module(&[&[OP_DECORATE, 1]])), Err(Fault::IndexOutOfRange));
        assert_eq!(read(&module(&[&[OP_TYPE_VECTOR, 2, 2, 4], &[OP_TYPE_STRUCT, 3, 2], &[OP_VARIABLE, 3, 9, STORAGE_UNIFORM]])), Err(Fault::Endless));
    }

    #[test]
    fn an_input_no_function_reads_leaves_the_entry_point_and_one_that_is_read_stays() {
        let mut entry = vec![OP_ENTRY_POINT, 4, 50];
        entry.extend(name("main"));
        entry.extend([60, 61, 62]);
        let spirv = module(&[&entry, &[OP_VARIABLE, 7, 60, STORAGE_INPUT], &[OP_VARIABLE, 7, 61, STORAGE_INPUT], &[OP_VARIABLE, 7, 62, 3], &[OP_FUNCTION, 1, 50, 0, 2], &[61, 8, 61]]);
        let pruned = without_unread_inputs(&spirv).unwrap().unwrap();
        let words = words_of(&pruned);
        assert_eq!(words[5], (7 << 16) | OP_ENTRY_POINT);
        assert_eq!(&words[10..12], [61, 62]);
        assert_eq!(pruned.len(), spirv.len() - 4);
        assert_eq!(without_unread_inputs(&pruned), Ok(Some(pruned.clone())), "the variable is still declared, so it is left out again");
        assert_eq!(without_unread_inputs(&[1, 2, 3]), Ok(None));
        let cut = module(&[&[OP_ENTRY_POINT, 4, 50], &[OP_VARIABLE, 7, 60, STORAGE_INPUT]]);
        assert_eq!(without_unread_inputs(&cut), Err(Fault::ArgumentOutOfRange));
    }

    #[test]
    fn two_stages_merge_by_name_the_first_of_each_kept() {
        let block = |binding, size, names: &[&str]| Some(Block { binding, size, members: names.iter().map(|n| Member { name: n.to_string(), offset: 0, size: 4 }).collect() });
        let vertex = Reflection { uniforms: block(1, 16, &["MVP", "a"]), push_constants: None, samplers: vec![Sampler { name: "Source".into(), binding: 2 }] };
        let fragment = Reflection { uniforms: block(9, 32, &["a", "b"]), push_constants: block(0, 8, &["p"]), samplers: vec![Sampler { name: "Source".into(), binding: 5 }, Sampler { name: "Mask".into(), binding: 3 }] };
        let merged = merge(&vertex, &fragment);
        let uniforms = merged.uniforms.unwrap();
        assert_eq!((uniforms.binding, uniforms.size, uniforms.members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>()), (1, 32, vec!["MVP", "a", "b"]));
        assert_eq!(merged.push_constants.unwrap().members.len(), 1);
        assert_eq!(merged.samplers, [Sampler { name: "Source".into(), binding: 2 }, Sampler { name: "Mask".into(), binding: 3 }]);
    }
}
