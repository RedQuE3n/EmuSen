//! A breakpoint's condition and logpoint, the C# `BreakpointExpressions` (`man bp`): both checked against the
//! registry's scope before anything is added, then evaluated at each hit by an [`crate::breakpoints::Evaluate`] over
//! that scope, its registers refreshed once an evaluation first names a symbol.

use crate::expr::{Context, Error, Evaluator};
use emusen_dotnet::text::trim;

/// A logpoint's comma-separated parts, each trimmed, the empty ones dropped.
pub fn parts(log: &str) -> impl Iterator<Item = &str> {
    log.split(',').map(trim).filter(|p| !p.is_empty())
}

fn check(context: &mut dyn Context, expression: &str, what: &str, whole: &str) -> Result<(), String> {
    match Evaluator::new(context).check(expression) {
        Ok(()) => Ok(()),
        Err(Error::Expression(words)) => Err(format!("{what} '{whole}' refused: {words} Nothing was added.")),
        Err(e) => Err(e.message().to_string()),
    }
}

/// `Prepare`: both expressions checked in the scope; true when either is given, so the registry's evaluators are set.
pub fn prepare(scope: &mut dyn Context, condition: Option<&str>, log: Option<&str>) -> Result<bool, String> {
    if condition.is_none() && log.is_none() {
        return Ok(false);
    }
    if let Some(c) = condition {
        check(scope, c, "Condition", c)?;
    }
    if let Some(l) = log {
        for part in parts(l) {
            check(scope, part, "Log expression", l)?;
        }
    }
    Ok(true)
}

/// `Live.Condition`: the condition's truth, or the words of its failure.
pub fn condition(live: &mut dyn Context, condition: &str) -> Result<bool, String> {
    Evaluator::new(live).evaluate_bool(condition).map_err(|e| e.message().to_string())
}

/// `Live.Render`: each part as `expr=0xVALUE`, two spaces apart, or the first failing part's words.
pub fn render(live: &mut dyn Context, log: &str) -> Result<String, String> {
    let mut evaluator = Evaluator::new(live);
    let mut rendered = Vec::new();
    for part in parts(log) {
        match evaluator.evaluate(part) {
            Ok(v) => rendered.push(format!("{part}=0x{v:X}")),
            Err(e) => return Err(format!("{part}: {}", e.message())),
        }
    }
    Ok(rendered.join("  "))
}
