use std::sync::Arc;
use std::time;
use arboard::Clipboard;
use eframe::{egui, NativeOptions};
use egui::IconData;
use image::io::Reader as ImageReader;
use std::io::Cursor;

/// Estructura principal de la aplicación del portapapeles
struct ClipboardApp {
    // Historial de textos copiados, con un máximo de 10 elementos
    history: Vec<String>,
}

impl ClipboardApp {
    /// Crea una nueva instancia de la aplicación con un historial vacío
    fn new() -> Self {
        Self { history: Vec::new() }
    }

    /// Verifica si hay nuevos textos copiados y los agrega al historial
    fn update_clipboard(&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        if let Ok(content) = clipboard.get_text() {
            // Solo agrega al historial si el texto es nuevo
            if self.history.first() != Some(&content) {
                self.history.insert(0, content);
                if self.history.len() > 10 {
                    self.history.pop(); // Mantiene solo los últimos 10 elementos
                }
            }
        }
    }

    /// Trunca los textos largos para que se muestren de manera ordenada en la UI
    fn truncate_text(text: &str, max_length: usize) -> String {
        if text.len() > max_length {
            format!("{}...", &text[..max_length])
        } else {
            text.to_string()
        }
    }
}

impl eframe::App for ClipboardApp {
    /// Método principal que renderiza la interfaz gráfica y gestiona la lógica de la aplicación
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_clipboard();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 Historial del Portapapeles");
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for text in &self.history {
                    let truncated_text = ClipboardApp::truncate_text(text, 50);
                    
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), 100.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let response = ui.add_sized(
                                [ui.available_width(), 80.0],
                                egui::Button::new(egui::RichText::new(truncated_text.clone()).strong().size(14.0)),
                            );
                            if response.clicked() {
                                let mut clipboard = Clipboard::new().unwrap();
                                clipboard.set_text(text.clone()).unwrap();
                            }
                        },
                    );
                    ui.separator();
                }
            });
            
            ui.label("Selecciona un texto para copiarlo");
        });
        ctx.request_repaint_after(time::Duration::from_secs(1)); // Redibuja la UI cada segundo
    }
}

/// Carga un ícono desde un archivo PNG en memoria
fn load_icon() -> Option<Arc<IconData>> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = ImageReader::new(Cursor::new(icon_bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Some(Arc::new(IconData { rgba, width, height }))
}

fn main() -> Result<(), eframe::Error> {
    let icon = load_icon();

    let viewport_builder = if let Some(icon_data) = icon {
        egui::ViewportBuilder::default().with_icon(icon_data)
    } else {
        egui::ViewportBuilder::default()
    };

    let options = NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        "📋 Clipboard Monitor", 
        options, 
        Box::new(|_cc| Box::new(ClipboardApp::new()))
    )
}