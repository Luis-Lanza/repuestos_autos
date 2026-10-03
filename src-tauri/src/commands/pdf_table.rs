//! Shared built-in Helvetica measurement and wrapping for PDF report tables.

/// Measure Helvetica Type 1 WinAnsi advances in points, matching printpdf's built-in renderer.
pub fn measure_helvetica_pt(text: &str, font_size_pt: f32) -> f32 {
    text.chars().map(|ch| helvetica_width(ch)).sum::<f32>() * font_size_pt / 1000.0
}

/// Wrap at word boundaries, splitting only tokens wider than the available cell.
pub fn wrap_helvetica(text: &str, max_width_pt: f32, font_size_pt: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        wrap_paragraph(
            paragraph.strip_suffix('\r').unwrap_or(paragraph),
            max_width_pt,
            font_size_pt,
            &mut lines,
        );
    }
    lines
}

fn wrap_paragraph(text: &str, max_width_pt: f32, font_size_pt: f32, lines: &mut Vec<String>) {
    if text.is_empty() {
        lines.push(String::new());
        return;
    }
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if measure_helvetica_pt(&candidate, font_size_pt) <= max_width_pt {
            line = candidate;
        } else {
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            let mut token = String::new();
            for ch in word.chars() {
                let candidate = format!("{token}{ch}");
                if !token.is_empty()
                    && measure_helvetica_pt(&candidate, font_size_pt) > max_width_pt
                {
                    lines.push(std::mem::take(&mut token));
                }
                token.push(ch);
            }
            line = token;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_words_at_boundaries_and_splits_only_long_tokens() {
        assert_eq!(wrap_helvetica("uno dos", 18.0, 9.0), vec!["uno", "dos"]);
        assert_eq!(
            wrap_helvetica("abcdefgh", 15.0, 9.0),
            vec!["abc", "def", "gh"]
        );
    }

    #[test]
    fn reports_independent_helvetica_base14_winansi_advances() {
        assert_eq!(measure_helvetica_pt("É", 1000.0), 667.0);
        assert_eq!(measure_helvetica_pt("Ç", 1000.0), 722.0);
        assert_eq!(measure_helvetica_pt("·", 1000.0), 278.0);
        assert_eq!(measure_helvetica_pt("º", 1000.0), 300.0);
        assert_eq!(measure_helvetica_pt("0123456789", 1000.0), 5560.0);
        assert_eq!(measure_helvetica_pt(".", 1000.0), 278.0);
        assert_eq!(measure_helvetica_pt(",", 1000.0), 278.0);
        assert_eq!(measure_helvetica_pt("+", 1000.0), 584.0);
        assert_eq!(measure_helvetica_pt("-", 1000.0), 333.0);
    }

    #[test]
    fn wraps_accented_words_and_preserves_hard_lines_and_blank_paragraphs() {
        assert_eq!(
            wrap_helvetica("niño café\n\nÉpoca", 30.0, 9.0),
            vec!["niño", "café", "", "Época"]
        );
    }
}

fn helvetica_width(ch: char) -> f32 {
    const ASCII: [u16; 95] = [
        278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556,
        556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722,
        722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722,
        667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 222, 556, 556, 500, 556, 556, 278, 556,
        556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500,
        500, 334, 260, 334, 584,
    ];
    let width = match ch {
        ' '..='~' => ASCII[ch as usize - 32] as f32,
        // Standard Type 1 Helvetica/WinAnsi advances; unsupported characters serialize as '?' (556).
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => 667.0,
        'Æ' => 1000.0,
        'Ç' => 722.0,
        'È' | 'É' | 'Ê' | 'Ë' => 667.0,
        'Ì' | 'Í' | 'Î' | 'Ï' => 278.0,
        'Ð' | 'Ñ' => 722.0,
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => 778.0,
        'Ù' | 'Ú' | 'Û' | 'Ü' => 722.0,
        'Ý' | 'Þ' => 667.0,
        'ß' => 611.0,
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 556.0,
        'æ' => 889.0,
        'ç' | 'è' | 'é' | 'ê' | 'ë' => 556.0,
        'ì' | 'í' | 'î' | 'ï' => 222.0,
        'ð' | 'ñ' => 556.0,
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 611.0,
        'ù' | 'ú' | 'û' | 'ü' => 556.0,
        'ý' | 'ÿ' => 500.0,
        'þ' => 556.0,
        '¡' => 333.0,
        '¢' | '£' | '¤' | '¥' | 'µ' => 556.0,
        '¦' => 260.0,
        '§' => 556.0,
        '¨' | '´' | '¸' => 333.0,
        '©' | '®' => 737.0,
        'ª' => 276.0,
        '«' | '»' => 333.0,
        '¬' => 584.0,
        '¯' => 333.0,
        '°' => 400.0,
        '±' => 584.0,
        '²' | '³' | '¹' => 333.0,
        '¶' => 537.0,
        '·' => 278.0,
        'º' => 300.0,
        '¼' | '½' | '¾' => 834.0,
        '¿' => 611.0,
        '×' | '÷' => 584.0,
        '‚' => 222.0,
        'ƒ' => 556.0,
        '„' => 333.0,
        '…' => 1000.0,
        '†' | '‡' => 556.0,
        'ˆ' => 333.0,
        '‰' => 1000.0,
        'Š' => 667.0,
        '‹' => 222.0,
        'Œ' => 1000.0,
        'Ž' => 611.0,
        '‘' | '’' => 222.0,
        '“' | '”' => 333.0,
        '•' => 350.0,
        '–' => 556.0,
        '—' => 1000.0,
        '˜' => 333.0,
        '™' => 1000.0,
        'š' => 500.0,
        '›' => 222.0,
        'œ' => 944.0,
        'ž' => 500.0,
        'Ÿ' => 667.0,
        '€' => 556.0,
        // printpdf serializes non-WinAnsi characters as '?' (556 units).
        _ => 556.0,
    };
    width
}
