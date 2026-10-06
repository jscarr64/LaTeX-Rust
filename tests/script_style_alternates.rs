// SPDX-License-Identifier: MIT OR Apache-2.0

use latex_rust::{layout, parse, BoxContent, MathFont, MathStyle};

fn ssty_alternate(face: &ttf_parser::Face<'_>, glyph_id: u16, script_level: u8) -> Option<u16> {
    let alternate_index = match script_level {
        1 => 0,
        2 => 1,
        _ => return None,
    };
    let gsub = face.tables().gsub?;
    let feature = gsub.features.find(ttf_parser::Tag::from_bytes(b"ssty"))?;
    for lookup_index in feature.lookup_indices {
        let Some(lookup) = gsub.lookups.get(lookup_index) else {
            continue;
        };
        for subtable in lookup
            .subtables
            .into_iter::<ttf_parser::gsub::SubstitutionSubtable<'_>>()
        {
            let ttf_parser::gsub::SubstitutionSubtable::Alternate(alternate) = subtable else {
                continue;
            };
            let Some(coverage_index) = alternate.coverage.get(ttf_parser::GlyphId(glyph_id)) else {
                continue;
            };
            let Some(set) = alternate.alternate_sets.get(coverage_index) else {
                continue;
            };
            if let Some(selected) = set.alternates.get(alternate_index) {
                return Some(selected.0);
            }
        }
    }
    None
}

fn glyph_id(source: &str, style: MathStyle, font: &MathFont) -> u16 {
    let ast = parse(source).expect("parse glyph case");
    let bx = layout(&ast, font, style).expect("layout glyph case");
    let BoxContent::Glyph { glyph_id, .. } = bx.content else {
        panic!("single glyph case must stay a glyph box");
    };
    glyph_id
}

#[test]
fn script_styles_use_open_type_ssty_alternates() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    let face = ttf_parser::Face::parse(font.bytes(), 0).expect("parse embedded font");
    let base = font.glyph('2').expect("digit 2").glyph_id;
    let script = ssty_alternate(&face, base, 1).expect("STIX script alternate for digit 2");
    let scriptscript =
        ssty_alternate(&face, base, 2).expect("STIX scriptscript alternate for digit 2");

    assert_ne!(script, base);
    assert_ne!(scriptscript, base);
    assert_eq!(glyph_id("2", MathStyle::Text, &font), base);
    assert_eq!(glyph_id("2", MathStyle::Script, &font), script);
    assert_eq!(glyph_id("2", MathStyle::ScriptScript, &font), scriptscript);
}
