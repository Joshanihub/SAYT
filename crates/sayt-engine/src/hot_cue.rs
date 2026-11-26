//! Validated cue metadata shared by both decks and the local cue store.

use serde::{Deserialize, Serialize};

pub const HOT_CUE_SLOT_COUNT: usize = 8;
pub const HOT_CUE_NAME_MAX_CHARS: usize = 32;
pub type HotCueSlots = [Option<HotCue>; HOT_CUE_SLOT_COUNT];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HotCueColor {
    Coral,
    Orange,
    Amber,
    Lime,
    Cyan,
    Blue,
    Violet,
    Pink,
}

impl HotCueColor {
    pub fn for_slot(slot: usize) -> Self {
        match slot % HOT_CUE_SLOT_COUNT {
            0 => Self::Coral,
            1 => Self::Orange,
            2 => Self::Amber,
            3 => Self::Lime,
            4 => Self::Cyan,
            5 => Self::Blue,
            6 => Self::Violet,
            _ => Self::Pink,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HotCue {
    pub name: String,
    pub color: HotCueColor,
    pub position_seconds: f64,
}

pub fn empty_hot_cue_slots() -> HotCueSlots {
    std::array::from_fn(|_| None)
}

pub fn default_hot_cue(slot: usize, position_seconds: f64) -> HotCue {
    HotCue {
        name: format!("Hot Cue {}", slot + 1),
        color: HotCueColor::for_slot(slot),
        position_seconds,
    }
}

pub fn validate_hot_cue_slots(slots: &HotCueSlots, duration_seconds: f64) -> Result<(), String> {
    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Err("The loaded track has an invalid duration for hot cues".to_string());
    }

    for (slot, cue) in slots.iter().enumerate() {
        let Some(cue) = cue else {
            continue;
        };
        validate_hot_cue(slot, cue, duration_seconds)?;
    }
    Ok(())
}

pub fn validate_hot_cue(slot: usize, cue: &HotCue, duration_seconds: f64) -> Result<(), String> {
    if slot >= HOT_CUE_SLOT_COUNT {
        return Err(format!(
            "Hot cue slot must be between 1 and {HOT_CUE_SLOT_COUNT}"
        ));
    }
    if !cue.position_seconds.is_finite()
        || cue.position_seconds < 0.0
        || cue.position_seconds > duration_seconds
    {
        return Err("Hot cue position is outside the loaded track".to_string());
    }

    let name = cue.name.trim();
    let character_count = name.chars().count();
    if character_count == 0 || character_count > HOT_CUE_NAME_MAX_CHARS {
        return Err(format!(
            "Hot cue names must contain 1 to {HOT_CUE_NAME_MAX_CHARS} characters"
        ));
    }
    if name.chars().any(char::is_control) {
        return Err("Hot cue names cannot contain control characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named_cue(name: &str, position_seconds: f64) -> HotCue {
        HotCue {
            name: name.to_string(),
            color: HotCueColor::Cyan,
            position_seconds,
        }
    }

    #[test]
    fn validates_empty_first_last_and_duration_boundary_slots() {
        let mut slots = empty_hot_cue_slots();
        slots[0] = Some(default_hot_cue(0, 0.0));
        slots[HOT_CUE_SLOT_COUNT - 1] = Some(default_hot_cue(7, 10.0));

        assert!(validate_hot_cue_slots(&slots, 10.0).is_ok());
    }

    #[test]
    fn rejects_invalid_positions_and_names() {
        assert!(validate_hot_cue(0, &named_cue("Before", -0.01), 10.0).is_err());
        assert!(validate_hot_cue(0, &named_cue("After", 10.01), 10.0).is_err());
        assert!(validate_hot_cue(0, &named_cue("NaN", f64::NAN), 10.0).is_err());
        assert!(validate_hot_cue(8, &named_cue("Slot", 0.0), 10.0).is_err());
        assert!(validate_hot_cue(0, &named_cue("  ", 0.0), 10.0).is_err());
        assert!(validate_hot_cue(0, &named_cue("Line\nBreak", 0.0), 10.0).is_err());
        assert!(validate_hot_cue(0, &named_cue(&"x".repeat(33), 0.0), 10.0).is_err());
    }

    #[test]
    fn accepts_unicode_names_within_the_character_limit() {
        assert!(validate_hot_cue(0, &named_cue("مطلع", 0.0), 10.0).is_ok());
    }

    #[test]
    fn default_colors_are_stable_and_distinct_by_slot() {
        let colors = (0..HOT_CUE_SLOT_COUNT)
            .map(HotCueColor::for_slot)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(colors.len(), HOT_CUE_SLOT_COUNT);
        assert_eq!(default_hot_cue(0, 0.0).name, "Hot Cue 1");
    }
}
