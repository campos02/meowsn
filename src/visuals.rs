// Modified version of catppuccin_egui theme (Latte and Mocha) https://crates.io/crates/catppuccin-egui

use eframe::egui::{Color32, style};
use eframe::{egui, epaint};

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

pub fn light_mode(old: egui::Visuals) -> egui::Visuals {
    let shadow_color = Color32::from_black_alpha(25);
    let blue = Color32::from_rgb(30, 102, 245);
    let surface0 = Color32::from_rgb(204, 208, 218);
    let surface1 = Color32::from_rgb(188, 192, 204);
    let surface2 = Color32::from_rgb(172, 176, 190);
    let crust = Color32::from_rgb(220, 224, 232);
    let mantle = Color32::from_rgb(230, 233, 239);
    let peach = Color32::from_rgb(254, 100, 11);
    let maroon = Color32::from_rgb(230, 69, 83);
    let base = Color32::from_rgb(239, 241, 245);
    let overlay1 = Color32::from_rgb(140, 143, 161);
    let text = Color32::from_rgb(76, 79, 105);

    egui::Visuals {
        hyperlink_color: blue,
        faint_bg_color: surface0,
        extreme_bg_color: crust,
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
            inactive: make_widget_visual(old.widgets.inactive, overlay1, text, surface0),
            hovered: make_widget_visual(old.widgets.hovered, overlay1, text, surface2),
            active: make_widget_visual(old.widgets.active, overlay1, text, surface1),
            open: make_widget_visual(old.widgets.open, overlay1, text, surface0),
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
        dark_mode: false,
        ..old
    }
}

pub fn dark_mode(old: egui::Visuals) -> egui::Visuals {
    let shadow_color = Color32::from_black_alpha(96);
    let blue = Color32::from_rgb(137, 180, 250);
    let surface0 = Color32::from_rgb(49, 50, 68);
    let surface1 = Color32::from_rgb(69, 71, 90);
    let surface2 = Color32::from_rgb(88, 91, 112);
    let crust = Color32::from_rgb(17, 17, 27);
    let mantle = Color32::from_rgb(24, 24, 37);
    let peach = Color32::from_rgb(250, 179, 135);
    let maroon = Color32::from_rgb(235, 160, 172);
    let base = Color32::from_rgb(30, 30, 46);
    let overlay1 = Color32::from_rgb(127, 132, 156);
    let text = Color32::from_rgb(205, 214, 244);

    egui::Visuals {
        hyperlink_color: blue,
        faint_bg_color: surface0,
        extreme_bg_color: crust,
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
            inactive: make_widget_visual(old.widgets.inactive, overlay1, text, surface0),
            hovered: make_widget_visual(old.widgets.hovered, overlay1, text, surface2),
            active: make_widget_visual(old.widgets.active, overlay1, text, surface1),
            open: make_widget_visual(old.widgets.open, overlay1, text, surface0),
        },
        selection: style::Selection {
            bg_fill: blue.linear_multiply(0.2),
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
        ..old
    }
}
