use eframe::egui::{self, Color32};

pub const ACCENT: Color32 = Color32::from_rgb(0, 191, 255);
pub const GOLD: Color32 = Color32::from_rgb(255, 215, 0);
pub const PURPLE: Color32 = Color32::from_rgb(102, 51, 153);
const INK: Color32 = Color32::from_rgb(22, 25, 40);

pub fn apply_theme(ctx: &egui::Context, dark: bool) {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = if dark {
        Color32::from_rgb(17, 24, 39)
    } else {
        Color32::from_rgb(238, 248, 255)
    };
    visuals.window_fill = if dark {
        Color32::from_rgb(28, 37, 54)
    } else {
        Color32::WHITE
    };
    visuals.widgets.noninteractive.fg_stroke.color = if dark {
        Color32::from_rgb(237, 242, 250)
    } else {
        INK
    };
    visuals.selection.bg_fill = PURPLE;
    visuals.selection.stroke = egui::Stroke::new(2.0_f32, GOLD);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.bg_fill = PURPLE;
        widget.weak_bg_fill = PURPLE;
        widget.fg_stroke = egui::Stroke::new(1.0_f32, Color32::WHITE);
        widget.bg_stroke = egui::Stroke::new(1.0_f32, if dark { GOLD } else { PURPLE });
        widget.corner_radius = egui::CornerRadius::same(8);
    }
    visuals.hyperlink_color = if dark { ACCENT } else { PURPLE };
    ctx.set_visuals(visuals);
    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(12.0, 10.0);
        style.spacing.button_padding = egui::vec2(16.0, 10.0);
        style.spacing.interact_size.y = 38.0;
    });
}

pub fn muted(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(185, 199, 217)
    } else {
        Color32::from_rgb(68, 78, 95)
    }
}
pub fn warning(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode { GOLD } else { PURPLE }
}
pub fn danger(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(255, 155, 155)
    } else {
        Color32::from_rgb(160, 24, 48)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contrast(a: Color32, b: Color32) -> f64 {
        fn luminance(c: Color32) -> f64 {
            let linear = |v: u8| {
                let v = f64::from(v) / 255.0;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
        }
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn text_pairs_meet_normal_text_contrast() {
        for dark in [false, true] {
            let ctx = egui::Context::default();
            apply_theme(&ctx, dark);
            let v = ctx.style().visuals.clone();
            for bg in [v.panel_fill, v.window_fill] {
                assert!(contrast(v.text_color(), bg) >= 4.5);
                let _ = ctx.run(egui::RawInput::default(), |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        for fg in [
                            muted(ui),
                            warning(ui),
                            danger(ui),
                            if dark { ACCENT } else { PURPLE },
                        ] {
                            assert!(contrast(fg, bg) >= 4.5, "{fg:?} on {bg:?}");
                        }
                    });
                });
            }
            for widget in [v.widgets.inactive, v.widgets.hovered, v.widgets.active] {
                assert!(contrast(widget.fg_stroke.color, widget.bg_fill) >= 4.5);
                assert!(contrast(widget.fg_stroke.color, widget.weak_bg_fill) >= 4.5);
                assert!(contrast(widget.fg_stroke.color, v.selection.bg_fill) >= 4.5);
            }
            assert!(contrast(v.selection.stroke.color, v.selection.bg_fill) >= 4.5);
        }
        // Purple on cyan is used only by the 30-point bold brand heading.
        assert!(contrast(PURPLE, ACCENT) >= 3.0);
    }
}
