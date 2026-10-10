//! A preset's parameters as its passes declare them, first declaration winning, each with the preset's own value as its
//! default where it gives one: the C# `SlangParameters`. See EmuSen_Serenity.md §7.6.

use crate::Error;
use crate::preset::Preset;
use crate::source::{self, Parameter};

pub fn merge<'a>(sources: impl IntoIterator<Item = &'a [Parameter]>, overrides: &[(String, f32)]) -> Vec<Parameter> {
    let mut merged: Vec<Parameter> = Vec::new();
    for parameters in sources {
        for parameter in parameters {
            if merged.iter().any(|known| known.id == parameter.id) {
                continue;
            }
            let mut parameter = parameter.clone();
            if let Some((_, value)) = overrides.iter().find(|(id, _)| *id == parameter.id) {
                parameter.initial = *value;
            }
            merged.push(parameter);
        }
    }
    merged
}

/// Reads every pass's source without compiling anything, for a settings window that has no device.
pub fn read(preset: &Preset) -> Result<Vec<Parameter>, Error> {
    let mut sources = Vec::new();
    for pass in &preset.passes {
        sources.push(source::load(&pass.shader_path)?);
    }
    Ok(merge(sources.iter().map(|source| source.parameters.as_slice()), &preset.parameters))
}

/// A parameter whose range has no width can only be a heading, which is how many presets label their groups.
pub fn is_heading(parameter: &Parameter) -> bool {
    parameter.maximum <= parameter.minimum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parameter(id: &str, initial: f32, minimum: f32, maximum: f32) -> Parameter {
        Parameter { id: id.to_string(), description: id.to_uppercase(), initial, minimum, maximum, step: 0.0 }
    }

    #[test]
    fn the_first_declaration_wins_and_the_preset_gives_its_value() {
        let first = [parameter("a", 1.0, 0.0, 2.0), parameter("b", 5.0, 0.0, 9.0)];
        let second = [parameter("b", 7.0, 1.0, 8.0), parameter("c", 0.0, 0.0, 0.0)];
        let merged = merge([&first[..], &second[..]], &[("b".to_string(), 6.0), ("z".to_string(), 1.0)]);
        assert_eq!(merged.iter().map(|p| (p.id.as_str(), p.initial, p.maximum)).collect::<Vec<_>>(), [("a", 1.0, 2.0), ("b", 6.0, 9.0), ("c", 0.0, 0.0)]);
        assert!(is_heading(&merged[2]) && !is_heading(&merged[0]));
        assert!(!is_heading(&parameter("n", 0.0, f32::NAN, 1.0)));
    }
}
