use arboard::Clipboard;
use daemonize::Daemonize;
use eframe::{egui, NativeOptions};
use egui::IconData;
use image::io::Reader as ImageReader;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, thread};

/// Ruta donde se guarda el historial del portapapeles
const HISTORY_FILE: &str = "/tmp/clipboard_history.json";

/// Estructura principal de la aplicación del portapapeles
struct ClipboardApp {
    history: Arc<Mutex<Vec<String>>>,
}

impl ClipboardApp {
    /// Crea una nueva instancia de la aplicación con un historial vacío o cargado desde un archivo
    fn new() -> Self {
        let history = Arc::new(Mutex::new(Self::load_history()));

        Self { history }
    }

    /// Carga el historial desde un archivo JSON
    fn load_history() -> Vec<String> {
        if let Ok(mut file) = File::open(HISTORY_FILE) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(history) = serde_json::from_str::<Vec<String>>(&contents) {
                    return history;
                }
            }
        }
        Vec::new()
    }

    /// Guarda el historial en un archivo JSON
    fn save_history(history: &Vec<String>) {
        if let Ok(mut file) = File::create(HISTORY_FILE) {
            let _ = file.write_all(serde_json::to_string(history).unwrap().as_bytes());
        }
    }

    /// Trunca los textos largos asegurándose de no cortar caracteres UTF-8
    fn truncate_text(text: &str, max_length: usize) -> String {
        let mut end = max_length;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        if text.len() > max_length {
            format!("{}...", &text[..end])
        } else {
            text.to_string()
        }
    }
}

/// Lógica de la UI
impl eframe::App for ClipboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let history = self.history.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 Historial del Portapapeles");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for text in history.iter() {
                    let truncated_text = ClipboardApp::truncate_text(text, 50);

                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), 100.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let response = ui.add_sized(
                                [ui.available_width(), 80.0],
                                egui::Button::new(
                                    egui::RichText::new(truncated_text.clone())
                                        .strong()
                                        .size(14.0),
                                ),
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
        ctx.request_repaint_after(Duration::from_secs(1)); // Redibuja la UI cada segundo
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

    Some(Arc::new(IconData {
        rgba,
        width,
        height,
    }))
}

/// Ejecuta el proceso en segundo plano como un daemon y guarda datos en un archivo
fn run_daemon() {
    let stdout = File::create("/tmp/clipboard-monitor.log").unwrap();
    let stderr = File::create("/tmp/clipboard-monitor.err").unwrap();

    let daemonize = Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .pid_file("/tmp/clipboard-monitor.pid");

    match daemonize.start() {
        Ok(_) => {
            let mut clipboard = Clipboard::new().unwrap();
            let mut history = ClipboardApp::load_history();

            loop {
                if let Ok(content) = clipboard.get_text() {
                    if history.first() != Some(&content) {
                        history.insert(0, content);
                        if history.len() > 10 {
                            history.pop();
                        }
                        ClipboardApp::save_history(&history);
                    }
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
        Err(e) => eprintln!("Error al iniciar el daemon: {}", e),
    }
}

/// Función principal que maneja tanto la UI como el daemon
fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--daemon".to_string()) {
        run_daemon();
        return Ok(());
    }

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
        Box::new(|_cc| Box::new(ClipboardApp::new())),
    )
}
