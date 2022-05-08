use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

fn material_properties_ui_system(
    global_state: Res<GlobalState>,
    mut egui_context: ResMut<EguiContext>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    egui::Window::new("Diagnostics").show(egui_context.ctx_mut(), |ui| {
        if let Some(material) = materials.get_mut(&global_state.tile_material) {
            let response = ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0));
            response.on_hover_text("metallic");

            let response = ui.add(egui::Slider::new(
                &mut material.perceptual_roughness,
                0.0..=1.0,
            ));
            response.on_hover_text("roughness");

            // let color: egui::Color32 = material.base_color.into();
            // ui.add(egui::color_picker::color_picker_color32(ui, srgba, alpha))
        }
    });
}
