use gpui::Hsla;
use ui::utils::ensure_minimum_contrast;

const RAINBOW_CARET_CONTRAST_CACHE_BUCKETS: usize = 240;
const RAINBOW_CARET_COLOR_QUANTIZATION: f32 = 4095.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct QuantizedHsla {
    h: u16,
    s: u16,
    l: u16,
    a: u16,
}

impl QuantizedHsla {
    fn new(color: Hsla) -> Self {
        Self {
            h: quantize_phase(color.h),
            s: quantize_unit(color.s),
            l: quantize_unit(color.l),
            a: quantize_unit(color.a),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct QuantizedRainbowForeground {
    h: u16,
    s: u16,
    l: u16,
    a: u16,
}

impl QuantizedRainbowForeground {
    fn new(color: Hsla) -> Self {
        Self {
            h: quantize_phase(color.h),
            s: quantize_unit(color.s),
            l: quantize_unit(color.l),
            a: quantize_unit(color.a),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RainbowCaretContrastCacheKey {
    foreground: QuantizedRainbowForeground,
    background: QuantizedHsla,
    minimum_apca_contrast: u32,
}

#[derive(Clone, Copy, Debug)]
struct RainbowCaretContrastCacheEntry {
    key: RainbowCaretContrastCacheKey,
    color: Hsla,
}

pub(crate) struct RainbowCaretContrastCache {
    entries: [Option<RainbowCaretContrastCacheEntry>; RAINBOW_CARET_CONTRAST_CACHE_BUCKETS],
}

impl RainbowCaretContrastCache {
    pub(crate) fn adjusted_color(
        &mut self,
        foreground: Hsla,
        background: Hsla,
        minimum_apca_contrast: f32,
    ) -> Hsla {
        let bucket = quantized_hue_bucket(foreground.h);
        let foreground = canonicalize_rainbow_foreground(foreground, bucket);
        let key = RainbowCaretContrastCacheKey {
            foreground: QuantizedRainbowForeground::new(foreground),
            background: QuantizedHsla::new(background),
            minimum_apca_contrast: minimum_apca_contrast.to_bits(),
        };

        if let Some(entry) = self.entries[bucket]
            && entry.key == key
        {
            return entry.color;
        }

        let color = ensure_minimum_contrast(foreground, background, minimum_apca_contrast);
        self.entries[bucket] = Some(RainbowCaretContrastCacheEntry { key, color });
        color
    }
}

impl Default for RainbowCaretContrastCache {
    fn default() -> Self {
        Self {
            entries: [None; RAINBOW_CARET_CONTRAST_CACHE_BUCKETS],
        }
    }
}

fn quantized_hue_bucket(hue: f32) -> usize {
    ((normalize_phase(hue) * RAINBOW_CARET_CONTRAST_CACHE_BUCKETS as f32).floor() as usize)
        .min(RAINBOW_CARET_CONTRAST_CACHE_BUCKETS - 1)
}

fn canonicalize_rainbow_foreground(color: Hsla, bucket: usize) -> Hsla {
    Hsla {
        h: (bucket as f32 + 0.5) / RAINBOW_CARET_CONTRAST_CACHE_BUCKETS as f32,
        s: clamp_unit(color.s),
        l: clamp_unit(color.l),
        a: clamp_unit(color.a),
    }
}

fn quantize_phase(value: f32) -> u16 {
    quantize_unit(normalize_phase(value))
}

fn quantize_unit(value: f32) -> u16 {
    (clamp_unit(value) * RAINBOW_CARET_COLOR_QUANTIZATION).round() as u16
}

fn normalize_phase(phase: f32) -> f32 {
    if phase.is_finite() {
        phase.rem_euclid(1.)
    } else {
        0.
    }
}

fn clamp_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0., 1.)
    } else {
        0.
    }
}
