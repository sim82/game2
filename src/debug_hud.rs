use std::{collections::VecDeque, time::Instant};

use bevy::{
    diagnostic::{
        DiagnosticId, Diagnostics, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    },
    prelude::*,
};
use bevy_egui::{
    egui::{self, plot},
    EguiContext,
};

use crate::property::{PropertyName, PropertyRegistry, PropertyUpdateEvent, PropertyValue};

fn mag_to_str(mag: i32) -> &'static str {
    match mag {
        0 => "",
        1 => "K",
        2 => "M",
        3 => "G",
        4 => "T",
        5 => "P",
        6 => "E",
        _ => "too large",
    }
}

pub fn hud_egui_setup_system(mut commands: Commands, mut hud_order: ResMut<HudOrder>) {
    let hud_group = "1. Diag";
    commands
        .spawn()
        .insert(HudElement::TextWithSource(HudSrc::Diagnostics(
            "FPS".into(),
            FrameTimeDiagnosticsPlugin::FPS,
            false,
        )))
        .insert(hud_order.next().in_group(hud_group));
    // commands
    //     .spawn()
    //     .insert(HudElement::TextWithSource(HudSrc::Diagnostics(
    //         "Int/s".into(),
    //         RAD_INT_PER_SECOND,
    //         true,
    //     )))
    //     .insert(hud_order.next().in_group(hud_group));
    // commands
    //     .spawn()
    //     .insert(HudElement::TextWithSource(HudSrc::RenderStatus))
    //     .insert(hud_order.next().in_group(hud_group));

    commands.spawn().insert(HudPlotDiagnostic::new(
        FrameTimeDiagnosticsPlugin::FPS,
        "fps",
    ));
    commands.spawn().insert(HudPlotDiagnostic::new(
        EntityCountDiagnosticsPlugin::ENTITY_COUNT,
        "entity count",
    ));
}

#[derive(Component)]
pub struct StringEdit {
    current_string: String,
}

#[derive(Clone, Debug)]
pub enum HudSrc {
    Diagnostics(String, DiagnosticId, bool),
    RenderStatus,
    LoadingScreen,
}
#[derive(Clone, Debug, Component)]
pub enum HudElement {
    TextWithSource(HudSrc),
    ToggleButtonProperty(String, String, String),
    EditThis,
}

#[derive(Component)]
pub struct HudPlotDiagnostic {
    pub(crate) id: DiagnosticId,
    pub(crate) name: String,
    pub(crate) buf: VecDeque<bevy_egui::egui::plot::Value>,
    pub(crate) x: f64,
}

impl HudPlotDiagnostic {
    pub fn new(id: DiagnosticId, name: &str) -> Self {
        HudPlotDiagnostic {
            id,
            name: name.to_string(),
            buf: VecDeque::new(),
            x: 0.0,
        }
    }
}

pub fn hud_egui_system(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    property_registry: Res<PropertyRegistry>,
    mut property_update_events: EventWriter<PropertyUpdateEvent>,
    property_query: Query<(&PropertyValue, &PropertyName)>,
    diagnostics: Res<Diagnostics>,
    // render_status: Res<RenderStatus>,
    hud_elements_query: Query<(Entity, &HudOrder, &HudElement)>,
    mut string_edit_query: Query<&mut StringEdit>,
    mut hud_plot_diagnostic: Query<&mut HudPlotDiagnostic>,
) {
    let mut ordered: Vec<_> = hud_elements_query.iter().collect();
    ordered.sort_by_key(|(_, o, _)| *o);

    let hud_groups = ordered.group_by(|(_, a, _), (_, b, _)| a.group == b.group);
    for c in hud_groups {
        if c.is_empty() {
            continue;
        }
        let title = c[0].1.group.clone().unwrap_or("HUD".to_string());
        egui::Window::new(&title).show(egui_context.ctx_mut(), |ui| {
            for (entity, _, element) in c {
                let entity = *entity; // hmm, is there a way to deref the whole thing?

                match element {
                    HudElement::TextWithSource(s) => {
                        let text = match s {
                            HudSrc::Diagnostics(diag_text, id, unit) => {
                                if let Some(fps) = diagnostics.get(*id) {
                                    let mut average = fps.average().unwrap_or_default();
                                    if *unit {
                                        let mut mag = 0;
                                        while average >= 1000f64 {
                                            average /= 1000f64;
                                            mag += 1;
                                        }

                                        format!("{} {:.3}{}", diag_text, average, mag_to_str(mag))
                                    } else {
                                        format!("{} {:.2}", diag_text, average)
                                    }
                                } else {
                                    format!("failed: {:?}", id)
                                }
                            }
                            HudSrc::RenderStatus => {
                                format!("render status: {}", "unknown")
                            }
                            HudSrc::LoadingScreen => String::new(),
                        };
                        ui.label(text);
                    }
                    HudElement::ToggleButtonProperty(property_name, _on_text, _off_text) => {
                        match property_registry.get(&property_name) {
                            Some(rs) => {
                                let (v, _) = property_query.get(rs).unwrap();
                                let v = match v {
                                    PropertyValue::Bool(v) => *v,
                                    _ => false,
                                };
                                if ui.button(format!("{}:{:?}", property_name, v)).clicked() {
                                    property_update_events.send(PropertyUpdateEvent::new(
                                        property_name.clone(),
                                        PropertyValue::Bool(!v),
                                    ));
                                }
                            }
                            _ => {
                                ui.label(format!("failed: {}", property_name));
                            }
                        }
                    }
                    HudElement::EditThis => match property_query.get(entity) {
                        Ok((property_value, property_name)) => {
                            match property_value {
                                PropertyValue::Bool(v) => {
                                    if ui.button(format!("{}:{:?}", property_name.0, v)).clicked() {
                                        property_update_events.send(PropertyUpdateEvent::new(
                                            property_name.0.clone(),
                                            PropertyValue::Bool(!v),
                                        ));
                                    }
                                }
                                PropertyValue::String(s) => match string_edit_query.get_mut(entity)
                                {
                                    Ok(mut string_edit) => {
                                        if ui
                                            .text_edit_singleline(&mut string_edit.current_string)
                                            .lost_focus()
                                        {
                                            commands.entity(entity).remove::<StringEdit>();
                                            property_update_events.send(PropertyUpdateEvent::new(
                                                property_name.0.clone(),
                                                PropertyValue::String(
                                                    string_edit.current_string.clone(),
                                                ),
                                            ));
                                        }
                                    }
                                    _ => {
                                        if ui.button(&property_name.0).clicked() {
                                            commands.entity(entity).insert(StringEdit {
                                                current_string: s.to_string(),
                                            });
                                        }
                                    }
                                },
                                PropertyValue::Color(color) => {
                                    let mut color = [color.x, color.y, color.z];
                                    if ui.color_edit_button_rgb(&mut color).changed() {
                                        property_update_events.send(PropertyUpdateEvent::new(
                                            property_name.0.clone(),
                                            PropertyValue::Color(color.into()),
                                        ));
                                    }
                                }
                                _ => (),
                            };
                        }
                        _ => {
                            ui.label(format!("failed: {:?}", entity));
                        }
                    },
                }
            }

            // ui.add(egui::TextEdit::singleline("text").hint_text("Write something here"));
        });
    }

    let mut plot_lines = Vec::new();

    for mut plot_diagnostic in hud_plot_diagnostic.iter_mut() {
        if let Some(diag) = diagnostics.get(plot_diagnostic.id) {
            let x = plot_diagnostic.x;
            plot_diagnostic.buf.push_back(egui::plot::Value::new(
                x,
                diag.value().unwrap_or_default(), /* * 1e-9*/
            ));
            plot_diagnostic.x += 1.0;
            if plot_diagnostic.buf.len() > 400 {
                plot_diagnostic.buf.pop_front();
            }

            let points = egui::plot::Line::new(egui::plot::Values::from_values_iter(
                plot_diagnostic.buf.iter().cloned(),
            ));
            plot_lines.push(points);
        }
    }

    egui::Window::new("plots").show(egui_context.ctx_mut(), |ui| {
        egui::plot::Plot::new("diag")
            .view_aspect(2.0)
            .show(ui, |plot_ui| {
                for points in plot_lines {
                    plot_ui.line(points);
                }
            });

        // for points in plot_lines {
        //     plot.
        // }
    });
}

pub fn hud_egui_plot_system(
    hud_plot_diagnostic: Query<&HudPlotDiagnostic>,
    diagnostics: Res<Diagnostics>,
    egui_context: Res<EguiContext>,
) {
    // for plot_diagnostic in hud_plot_diagnostic.iter() {
    //     if let Some(diag) = diagnostics.get(plot_diagnostic.id) {
    //         let points = egui::plot::Points::new(egui::plot::Values::from_values(
    //             diag.values()
    //                 .enumerate()
    //                 .map(|(i, v)| egui::plot::Value::new(i as f64, *v))
    //                 .collect(),
    //         ))
    //         .stems(-1.5)
    //         .radius(1.0);
    //         let plot = egui::plot::Plot::new("diag").points(points);
    //         egui::Window::new("diag").show(egui_context.ctx(), |ui| {
    //             ui.add(plot);
    //         });
    //     }
    // }
}

#[derive(Clone, Eq, PartialEq, Default, Ord, PartialOrd, Debug, Component)]
pub struct HudOrder {
    group: Option<String>,
    id: usize,
}

impl HudOrder {
    pub fn next(&mut self) -> HudOrder {
        let ret = self.clone();
        self.id += 1;
        ret
    }
    pub fn in_group(mut self, group: &str) -> HudOrder {
        self.group = Some(group.to_string());
        self
    }
}

#[derive(Default)]
pub struct DebugHudPlugin;

impl Plugin for DebugHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudOrder>()
            .add_startup_system(hud_egui_setup_system)
            .add_system(hud_egui_system)
            // .add_system(hud_egui_plot_system.system())
            ;
    }
}
