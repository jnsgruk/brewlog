/// Maps tasting note strings to colour categories based on the SCA Coffee
/// Tasting Wheel.  Unknown notes fall back to the neutral `pill-muted` style.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoteCategory {
    Floral,
    Fruity,
    Citrus,
    Sweet,
    Nutty,
    Spice,
    Roasted,
    Sour,
    Vegetal,
    Default,
}

impl NoteCategory {
    const fn pill_class(self) -> &'static str {
        match self {
            Self::Floral => "pill pill-floral",
            Self::Fruity => "pill pill-fruity",
            Self::Citrus => "pill pill-citrus",
            Self::Sweet => "pill pill-sweet",
            Self::Nutty => "pill pill-nutty",
            Self::Spice => "pill pill-spice",
            Self::Roasted => "pill pill-roasted",
            Self::Sour => "pill pill-sour",
            Self::Vegetal => "pill pill-vegetal",
            Self::Default => "pill pill-muted",
        }
    }
}

#[derive(Clone)]
pub struct TastingNoteView {
    pub label: String,
    pub pill_class: &'static str,
}

/// Categorise a tasting note string and return a view with the appropriate
/// pill class.  Matching is case-insensitive: first an exact match against
/// known SCA wheel terms, then a substring scan for common keywords, then
/// a fuzzy (Levenshtein distance) match for typo tolerance.
pub fn categorize(note: &str) -> TastingNoteView {
    let lower = note.to_lowercase();
    let category = exact_match(&lower)
        .or_else(|| substring_match(&lower))
        .or_else(|| fuzzy_match(&lower))
        .unwrap_or(NoteCategory::Default);
    TastingNoteView {
        label: note.to_string(),
        pill_class: category.pill_class(),
    }
}

/// Split raw tasting note strings on commas and newlines, trim whitespace,
/// drop empties, and categorise each resulting segment.
pub fn parse_and_categorize(notes: &[String]) -> Vec<TastingNoteView> {
    notes
        .iter()
        .flat_map(|note| {
            note.split([',', '\n'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .map(|n| categorize(&n))
        .collect()
}

// ── Exact matches ────────────────────────────────────────────────────

fn exact_match(lower: &str) -> Option<NoteCategory> {
    EXACT_MATCHES
        .iter()
        .find(|(term, _)| *term == lower)
        .map(|(_, cat)| *cat)
}

use NoteCategory::{Citrus, Floral, Fruity, Nutty, Roasted, Sour, Spice, Sweet, Vegetal};

const EXACT_MATCHES: &[(&str, NoteCategory)] = &[
    // Floral
    ("floral", Floral),
    ("jasmine", Floral),
    ("rose", Floral),
    ("chamomile", Floral),
    ("lavender", Floral),
    ("hibiscus", Floral),
    ("elderflower", Floral),
    ("violet", Floral),
    ("honeysuckle", Floral),
    ("orange blossom", Floral),
    // Berry
    ("berry", Fruity),
    ("blueberry", Fruity),
    ("strawberry", Fruity),
    ("raspberry", Fruity),
    ("blackberry", Fruity),
    ("cranberry", Fruity),
    ("boysenberry", Fruity),
    ("currant", Fruity),
    ("blackcurrant", Fruity),
    ("redcurrant", Fruity),
    ("red currant", Fruity),
    ("black currant", Fruity),
    // Dried fruit
    ("raisin", Fruity),
    ("prune", Fruity),
    ("fig", Fruity),
    ("date", Fruity),
    ("dried fruit", Fruity),
    // Other fruit
    ("cherry", Fruity),
    ("pomegranate", Fruity),
    ("pineapple", Fruity),
    ("grape", Fruity),
    ("apple", Fruity),
    ("red apple", Fruity),
    ("green apple", Fruity),
    ("peach", Fruity),
    ("pear", Fruity),
    ("plum", Fruity),
    ("apricot", Fruity),
    ("mango", Fruity),
    ("papaya", Fruity),
    ("guava", Fruity),
    ("passion fruit", Fruity),
    ("passionfruit", Fruity),
    ("coconut", Fruity),
    ("melon", Fruity),
    ("watermelon", Fruity),
    ("yellow fruit", Fruity),
    ("stone fruit", Fruity),
    ("tropical", Fruity),
    ("tropical fruit", Fruity),
    ("fruit", Fruity),
    ("fruity", Fruity),
    ("juicy", Fruity),
    ("tomato", Fruity),
    ("rhubarb", Fruity),
    // Citrus
    ("citrus", Citrus),
    ("lemon", Citrus),
    ("lime", Citrus),
    ("orange", Citrus),
    ("grapefruit", Citrus),
    ("bergamot", Citrus),
    ("tangerine", Citrus),
    ("mandarin", Citrus),
    ("yuzu", Citrus),
    ("clementine", Citrus),
    ("zesty", Citrus),
    ("citric", Citrus),
    ("citric acid", Citrus),
    // Sweet
    ("sweet", Sweet),
    ("caramel", Sweet),
    ("honey", Sweet),
    ("vanilla", Sweet),
    ("vanillin", Sweet),
    ("brown sugar", Sweet),
    ("chocolate", Sweet),
    ("dark chocolate", Sweet),
    ("milk chocolate", Sweet),
    ("white chocolate", Sweet),
    ("toffee", Sweet),
    ("butterscotch", Sweet),
    ("maple", Sweet),
    ("maple syrup", Sweet),
    ("molasses", Sweet),
    ("caramelized", Sweet),
    ("sugar cane", Sweet),
    ("sugarcane", Sweet),
    ("candy", Sweet),
    ("marshmallow", Sweet),
    ("nougat", Sweet),
    ("lychee", Sweet),
    ("syrupy", Sweet),
    ("black tea", Sweet),
    ("tea", Sweet),
    ("nasturtium", Sweet),
    ("fudge", Sweet),
    // Nutty / Cocoa
    ("nutty", Nutty),
    ("hazelnut", Nutty),
    ("almond", Nutty),
    ("peanut", Nutty),
    ("peanuts", Nutty),
    ("walnut", Nutty),
    ("pecan", Nutty),
    ("macadamia", Nutty),
    ("pistachio", Nutty),
    ("cashew", Nutty),
    ("cocoa", Nutty),
    ("cacao", Nutty),
    ("praline", Nutty),
    ("marzipan", Nutty),
    ("roasted almond", Nutty),
    ("roasted nuts", Nutty),
    // Spice
    ("spice", Spice),
    ("spicy", Spice),
    ("cinnamon", Spice),
    ("nutmeg", Spice),
    ("clove", Spice),
    ("anise", Spice),
    ("star anise", Spice),
    ("cardamom", Spice),
    ("ginger", Spice),
    ("pepper", Spice),
    ("black pepper", Spice),
    ("pink pepper", Spice),
    ("allspice", Spice),
    ("brown spice", Spice),
    ("pungent", Spice),
    // Roasted
    ("roasted", Roasted),
    ("smoky", Roasted),
    ("tobacco", Roasted),
    ("pipe tobacco", Roasted),
    ("ashy", Roasted),
    ("burnt", Roasted),
    ("charred", Roasted),
    ("malt", Roasted),
    ("grain", Roasted),
    ("cereal", Roasted),
    ("toast", Roasted),
    ("toasted", Roasted),
    ("roasty", Roasted),
    ("dark roast", Roasted),
    // Sour / Fermented
    ("sour", Sour),
    ("fermented", Sour),
    ("winey", Sour),
    ("wine", Sour),
    ("whiskey", Sour),
    ("boozy", Sour),
    ("acetic", Sour),
    ("acetic acid", Sour),
    ("malic acid", Sour),
    ("mead", Sour),
    ("tart", Sour),
    ("tangy", Sour),
    ("vinous", Sour),
    ("overripe", Sour),
    // Green / Vegetal
    ("green", Vegetal),
    ("vegetal", Vegetal),
    ("vegetative", Vegetal),
    ("herbal", Vegetal),
    ("grassy", Vegetal),
    ("hay", Vegetal),
    ("herb-like", Vegetal),
    ("fresh", Vegetal),
    ("earthy", Vegetal),
    ("woody", Vegetal),
    ("cedar", Vegetal),
    ("pine", Vegetal),
    ("mint", Vegetal),
    ("eucalyptus", Vegetal),
    ("sage", Vegetal),
    ("thyme", Vegetal),
    ("basil", Vegetal),
    ("yoghurt", Vegetal),
    ("yogurt", Vegetal),
    ("cream", Vegetal),
    ("creamy", Vegetal),
];

// ── Substring fallback ───────────────────────────────────────────────

fn substring_match(lower: &str) -> Option<NoteCategory> {
    // Order: specific before general to avoid false positives.
    const KEYWORDS: &[(&str, NoteCategory)] = &[
        // Floral
        ("floral", Floral),
        ("blossom", Floral),
        ("flower", Floral),
        ("jasmine", Floral),
        ("rose", Floral),
        // Fruity / Berry
        ("berry", Fruity),
        ("cherry", Fruity),
        ("plum", Fruity),
        ("peach", Fruity),
        ("apricot", Fruity),
        ("mango", Fruity),
        ("grape", Fruity),
        ("apple", Fruity),
        ("pear", Fruity),
        ("melon", Fruity),
        ("fruit", Fruity),
        ("tropical", Fruity),
        ("juicy", Fruity),
        ("raisin", Fruity),
        ("prune", Fruity),
        ("fig", Fruity),
        // Citrus
        ("citrus", Citrus),
        ("lemon", Citrus),
        ("lime", Citrus),
        ("grapefruit", Citrus),
        ("bergamot", Citrus),
        ("orange", Citrus),
        ("tangerine", Citrus),
        ("zesty", Citrus),
        // Sweet / Chocolate
        ("chocolate", Sweet),
        ("caramel", Sweet),
        ("honey", Sweet),
        ("vanilla", Sweet),
        ("toffee", Sweet),
        ("butterscotch", Sweet),
        ("maple", Sweet),
        ("molasses", Sweet),
        ("sugar", Sweet),
        ("candy", Sweet),
        ("syrup", Sweet),
        ("sweet", Sweet),
        // Nutty
        ("nut", Nutty),
        ("cocoa", Nutty),
        ("cacao", Nutty),
        ("praline", Nutty),
        ("marzipan", Nutty),
        ("almond", Nutty),
        // Spice
        ("cinnamon", Spice),
        ("clove", Spice),
        ("cardamom", Spice),
        ("ginger", Spice),
        ("pepper", Spice),
        ("spice", Spice),
        ("spicy", Spice),
        // Roasted
        ("smoke", Roasted),
        ("smoky", Roasted),
        ("tobacco", Roasted),
        ("roast", Roasted),
        ("toast", Roasted),
        ("malt", Roasted),
        ("grain", Roasted),
        ("ash", Roasted),
        ("burnt", Roasted),
        ("charred", Roasted),
        // Sour / Fermented
        ("wine", Sour),
        ("ferment", Sour),
        ("tart", Sour),
        ("sour", Sour),
        ("tangy", Sour),
        ("vinous", Sour),
        ("boozy", Sour),
        // Vegetal
        ("herbal", Vegetal),
        ("herb", Vegetal),
        ("grass", Vegetal),
        ("green", Vegetal),
        ("earthy", Vegetal),
        ("woody", Vegetal),
        ("cedar", Vegetal),
        ("pine", Vegetal),
        ("mint", Vegetal),
    ];

    for (keyword, category) in KEYWORDS {
        if lower.contains(keyword) {
            return Some(*category);
        }
    }

    None
}

// ── Fuzzy match using Levenshtein distance ───────────────────────────

fn levenshtein(a: &str, b: &str) -> usize {
    let b_chars: Vec<char> = b.chars().collect();
    let b_len = b_chars.len();

    if b_len == 0 {
        return a.chars().count();
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];

    for (i, a_char) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, &b_char) in b_chars.iter().enumerate() {
            let cost = usize::from(a_char != b_char);
            curr[j + 1] = (prev[j] + cost).min(curr[j] + 1).min(prev[j + 1] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

fn max_edit_distance(len: usize) -> usize {
    match len {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    }
}

/// Attempt to match the input against `EXACT_MATCHES` using Levenshtein
/// distance.  First tries the full input, then falls back to matching
/// individual words.  Returns `None` if no term is within threshold.
fn fuzzy_match(lower: &str) -> Option<NoteCategory> {
    // Try matching the full input string
    if let Some(cat) = best_fuzzy_hit(lower) {
        return Some(cat);
    }

    // Fall back to matching individual words
    for word in lower.split_whitespace() {
        if let Some(cat) = best_fuzzy_hit(word) {
            return Some(cat);
        }
    }

    None
}

fn best_fuzzy_hit(input: &str) -> Option<NoteCategory> {
    let threshold = max_edit_distance(input.len());
    if threshold == 0 {
        return None;
    }

    let mut best: Option<(usize, NoteCategory)> = None;
    for (term, category) in EXACT_MATCHES {
        let distance = levenshtein(input, term);
        if distance > 0 && distance <= threshold {
            match best {
                None => best = Some((distance, *category)),
                Some((d, _)) if distance < d => best = Some((distance, *category)),
                _ => {}
            }
        }
    }

    best.map(|(_, cat)| cat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_known_notes() {
        assert_eq!(categorize("Jasmine").pill_class, "pill pill-floral");
        assert_eq!(categorize("Blueberry").pill_class, "pill pill-fruity");
        assert_eq!(categorize("Lemon").pill_class, "pill pill-citrus");
        assert_eq!(categorize("Caramel").pill_class, "pill pill-sweet");
        assert_eq!(categorize("Hazelnut").pill_class, "pill pill-nutty");
        assert_eq!(categorize("Cinnamon").pill_class, "pill pill-spice");
        assert_eq!(categorize("Smoky").pill_class, "pill pill-roasted");
        assert_eq!(categorize("Winey").pill_class, "pill pill-sour");
        assert_eq!(categorize("Herbal").pill_class, "pill pill-vegetal");
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(categorize("JASMINE").pill_class, "pill pill-floral");
        assert_eq!(categorize("dark chocolate").pill_class, "pill pill-sweet");
    }

    #[test]
    fn substring_fallback() {
        assert_eq!(categorize("Wild Blueberry").pill_class, "pill pill-fruity");
        assert_eq!(categorize("Citrus Zest").pill_class, "pill pill-citrus");
        assert_eq!(categorize("Roasted Almond").pill_class, "pill pill-nutty");
    }

    #[test]
    fn unknown_note_falls_back_to_muted() {
        assert_eq!(categorize("Umami").pill_class, "pill pill-muted");
        assert_eq!(categorize("Complex").pill_class, "pill pill-muted");
    }

    #[test]
    fn preserves_original_label() {
        let view = categorize("Dark Chocolate");
        assert_eq!(view.label, "Dark Chocolate");
    }

    // ── Fuzzy matching ──────────────────────────────────────────────

    #[test]
    fn fuzzy_single_typo() {
        assert_eq!(categorize("cinamon").pill_class, "pill pill-spice");
        assert_eq!(categorize("smokey").pill_class, "pill pill-roasted");
        assert_eq!(categorize("rasberry").pill_class, "pill pill-fruity");
        assert_eq!(categorize("lemmon").pill_class, "pill pill-citrus");
    }

    #[test]
    fn fuzzy_multi_word_typo() {
        assert_eq!(categorize("pasion fruit").pill_class, "pill pill-fruity");
        assert_eq!(categorize("brown suger").pill_class, "pill pill-sweet");
    }

    #[test]
    fn fuzzy_word_by_word() {
        assert_eq!(categorize("wild cheery").pill_class, "pill pill-fruity");
    }

    #[test]
    fn fuzzy_skips_short_inputs() {
        assert_eq!(categorize("fif").pill_class, "pill pill-muted");
        assert_eq!(categorize("tee").pill_class, "pill pill-muted");
    }

    #[test]
    fn fuzzy_beyond_threshold() {
        assert_eq!(categorize("cinmn").pill_class, "pill pill-muted");
    }

    #[test]
    fn fuzzy_does_not_override_exact() {
        assert_eq!(categorize("cherry").pill_class, "pill pill-fruity");
        assert_eq!(categorize("smoky").pill_class, "pill pill-roasted");
    }

    #[test]
    fn fuzzy_does_not_override_substring() {
        assert_eq!(categorize("smokey notes").pill_class, "pill pill-roasted");
    }

    #[test]
    fn fuzzy_case_insensitive() {
        assert_eq!(categorize("CINAMON").pill_class, "pill pill-spice");
        assert_eq!(categorize("Smokey").pill_class, "pill pill-roasted");
    }

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("same", "same"), 0);
        assert_eq!(levenshtein("smokey", "smoky"), 1);
    }

    // ── parse_and_categorize ───────────────────────────────────────────

    #[test]
    fn parse_and_categorize_splits_commas() {
        let notes = vec!["chocolate, caramel".to_string()];
        let result = parse_and_categorize(&notes);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].label, "chocolate");
        assert_eq!(result[1].label, "caramel");
    }

    #[test]
    fn parse_and_categorize_splits_newlines() {
        let notes = vec!["chocolate\ncaramel".to_string()];
        let result = parse_and_categorize(&notes);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].label, "chocolate");
        assert_eq!(result[1].label, "caramel");
    }

    #[test]
    fn parse_and_categorize_filters_empty() {
        let notes = vec!["chocolate, , caramel".to_string()];
        let result = parse_and_categorize(&notes);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].label, "chocolate");
        assert_eq!(result[1].label, "caramel");
    }

    #[test]
    fn parse_and_categorize_empty_input() {
        let notes: Vec<String> = vec![];
        let result = parse_and_categorize(&notes);
        assert!(result.is_empty());
    }
}
