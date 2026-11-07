//! Input as the kit sees it: one event type for the pad, the pointer and the keyboard, the pad's grammar with its repeat tiers, and the frame's walk (EmuSen_Qt.md §1.7, EmuSen_Kit.md §2).

mod button;
mod event;
mod grammar;
mod walk;

pub use button::{Key, UiButton};
pub use event::{ElementId, InputEvent, Intent, PointerButton, intent};
pub use grammar::{FastTier, Grammar, Press, RepeatTiers, STICK_THRESHOLD, stick_direction};
pub use walk::{Role, Walk, WalkAction, WalkEvent, WalkForm};
