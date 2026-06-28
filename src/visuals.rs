use eframe::egui::{Color32, FontTweak, style};
use eframe::epaint::FontFamily;
use eframe::epaint::text::{FontData, FontDefinitions, VariationCoords};
use eframe::{egui, epaint};
use std::borrow::Cow;
use std::sync::Arc;

fn make_widget_visual(
    old: style::WidgetVisuals,
    overlay1: Color32,
    text: Color32,
    bg_fill: Color32,
) -> style::WidgetVisuals {
    style::WidgetVisuals {
        bg_fill,
        weak_bg_fill: bg_fill,
        bg_stroke: egui::Stroke {
            color: overlay1,
            ..old.bg_stroke
        },
        fg_stroke: egui::Stroke {
            color: text,
            ..old.fg_stroke
        },
        ..old
    }
}

// Modified version of catppuccin_egui theme (Latte and Mocha) https://crates.io/crates/catppuccin-egui
pub fn light_mode(old: egui::Visuals) -> egui::Visuals {
    let shadow_color = Color32::from_black_alpha(25);
    let blue = Color32::from_rgb(58, 117, 245);
    let surface0 = Color32::from_rgb(228, 228, 240);
    let surface1 = Color32::from_rgb(247, 247, 247);
    let mantle = Color32::from_rgb(230, 233, 239);
    let peach = Color32::from_rgb(254, 100, 11);
    let maroon = Color32::from_rgb(230, 69, 83);
    let base = Color32::from_rgb(228, 235, 242);
    let overlay1 = Color32::from_rgb(140, 143, 161);
    let text = Color32::from_rgb(77, 81, 107);

    egui::Visuals {
        hyperlink_color: blue,
        faint_bg_color: surface0,
        extreme_bg_color: surface1,
        code_bg_color: mantle,
        warn_fg_color: peach,
        error_fg_color: maroon,
        window_fill: base,
        panel_fill: base,
        window_stroke: egui::Stroke {
            color: overlay1,
            ..old.window_stroke
        },
        widgets: style::Widgets {
            noninteractive: make_widget_visual(old.widgets.noninteractive, overlay1, text, base),
            inactive: make_widget_visual(old.widgets.inactive, overlay1, text, surface1),
            hovered: make_widget_visual(old.widgets.hovered, overlay1, text, surface1),
            active: make_widget_visual(old.widgets.active, overlay1, text, surface0),
            open: make_widget_visual(old.widgets.open, overlay1, text, surface1),
        },
        selection: style::Selection {
            bg_fill: blue.linear_multiply(0.3),
            stroke: egui::Stroke {
                color: text,
                ..old.selection.stroke
            },
        },
        window_shadow: epaint::Shadow {
            color: shadow_color,
            ..old.window_shadow
        },
        popup_shadow: epaint::Shadow {
            color: shadow_color,
            ..old.popup_shadow
        },
        dark_mode: false,
        text_cursor: style::TextCursorStyle {
            stroke: egui::Stroke {
                color: text,
                ..old.text_cursor.stroke
            },
            ..old.text_cursor
        },
        ..old
    }
}

// Modified version of catppuccin_egui theme (Latte and Mocha) https://crates.io/crates/catppuccin-egui
pub fn dark_mode(old: egui::Visuals) -> egui::Visuals {
    let shadow_color = Color32::from_black_alpha(96);
    let blue = Color32::from_rgb(137, 180, 250);
    let surface0 = Color32::from_rgb(49, 50, 68);
    let surface1 = Color32::from_rgb(69, 71, 90);
    let surface2 = Color32::from_rgb(88, 91, 112);
    let mantle = Color32::from_rgb(24, 24, 37);
    let peach = Color32::from_rgb(250, 179, 135);
    let maroon = Color32::from_rgb(235, 160, 172);
    let base = Color32::from_rgb(30, 30, 46);
    let overlay1 = Color32::from_rgb(127, 132, 156);
    let text = Color32::from_rgb(205, 214, 244);

    egui::Visuals {
        hyperlink_color: blue,
        faint_bg_color: surface0,
        extreme_bg_color: surface1,
        code_bg_color: mantle,
        warn_fg_color: peach,
        error_fg_color: maroon,
        window_fill: base,
        panel_fill: base,
        window_stroke: egui::Stroke {
            color: overlay1,
            ..old.window_stroke
        },
        widgets: style::Widgets {
            noninteractive: make_widget_visual(old.widgets.noninteractive, overlay1, text, base),
            inactive: make_widget_visual(old.widgets.inactive, overlay1, text, surface1),
            hovered: make_widget_visual(old.widgets.hovered, overlay1, text, surface2),
            active: make_widget_visual(old.widgets.active, overlay1, text, surface1),
            open: make_widget_visual(old.widgets.open, overlay1, text, surface1),
        },
        selection: style::Selection {
            bg_fill: blue.linear_multiply(0.4),
            stroke: egui::Stroke {
                color: text,
                ..old.selection.stroke
            },
        },
        window_shadow: epaint::Shadow {
            color: shadow_color,
            ..old.window_shadow
        },
        popup_shadow: epaint::Shadow {
            color: shadow_color,
            ..old.popup_shadow
        },
        dark_mode: true,
        text_cursor: style::TextCursorStyle {
            stroke: egui::Stroke {
                color: text,
                ..old.text_cursor.stroke
            },
            ..old.text_cursor
        },
        ..old
    }
}

pub fn load_fonts() -> FontDefinitions {
    let mut font_definitions = FontDefinitions::default();

    let noto_sans = include_bytes!("../assets/fonts/NotoSans-VariableFont_wdth,wght.ttf");
    let noto_sans_cjk = include_bytes!("../assets/fonts/NotoSansCJK-VF.ttf.ttc");
    let noto_sans_arabic =
        include_bytes!("../assets/fonts/NotoSansArabic-VariableFont_wdth,wght.ttf");

    font_definitions.font_data.insert(
        "noto_sans".to_string(),
        Arc::new(FontData::from_static(noto_sans)),
    );

    font_definitions
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans".to_string());

    font_definitions.font_data.insert(
        "noto_sans_bold".to_string(),
        Arc::new(FontData {
            font: Cow::Borrowed(noto_sans),
            index: 0,
            tweak: FontTweak {
                coords: VariationCoords::new([(b"wght", 700.)]),
                ..Default::default()
            },
        }),
    );

    font_definitions
        .families
        .entry(FontFamily::Name("Bold".into()))
        .or_default()
        .insert(0, "noto_sans_bold".to_string());

    font_definitions.font_data.insert(
        "noto_sans_cjk".to_string(),
        Arc::new(FontData {
            font: Cow::Borrowed(noto_sans_cjk),
            index: 0,
            tweak: FontTweak {
                coords: VariationCoords::new([(b"wght", 400.)]),
                ..Default::default()
            },
        }),
    );

    font_definitions
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push("noto_sans_cjk".to_string());

    font_definitions.font_data.insert(
        "noto_sans_cjk_bold".to_string(),
        Arc::new(FontData {
            font: Cow::Borrowed(noto_sans_cjk),
            index: 0,
            tweak: FontTweak {
                coords: VariationCoords::new([(b"wght", 700.)]),
                ..Default::default()
            },
        }),
    );

    font_definitions
        .families
        .entry(FontFamily::Name("Bold".into()))
        .or_default()
        .push("noto_sans_cjk_bold".to_string());

    font_definitions.font_data.insert(
        "noto_sans_arabic".to_string(),
        Arc::new(FontData::from_static(noto_sans_arabic)),
    );

    font_definitions
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push("noto_sans_arabic".to_string());

    font_definitions.font_data.insert(
        "noto_sans_arabic_bold".to_string(),
        Arc::new(FontData {
            font: Cow::Borrowed(noto_sans_arabic),
            index: 0,
            tweak: FontTweak {
                coords: VariationCoords::new([(b"wght", 700.)]),
                ..Default::default()
            },
        }),
    );

    font_definitions
        .families
        .entry(FontFamily::Name("Bold".into()))
        .or_default()
        .push("noto_sans_arabic_bold".to_string());

    font_definitions
}
