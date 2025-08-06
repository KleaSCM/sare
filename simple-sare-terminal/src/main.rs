use anyhow::Result;
use eframe;
use egui;

fn main() -> Result<()> {
    println!("🚀 Starting Sare Terminal Emulator...");
    println!("💕 Built with love and passion by Yuriko and KleaSCM");
    
    let app = SimpleTerminalApp::default();
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    println!("🖼️  Starting GUI terminal window...");
    let run_result = eframe::run_native(
        "Sare Terminal Emulator",
        native_options,
        Box::new(|_cc| Box::new(app)),
    );
    
    match run_result {
        Ok(_) => {
            println!("✅ Sare Terminal Emulator completed successfully!");
        }
        Err(e) => {
            eprintln!("❌ Sare Terminal Emulator failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

struct SimpleTerminalApp {
    input_text: String,
    output_history: Vec<String>,
    current_dir: String,
}

impl Default for SimpleTerminalApp {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            output_history: vec![
                "🚀 Welcome to Sare Terminal Emulator!".to_string(),
                "💕 Built with love and passion by Yuriko and KleaSCM".to_string(),
                "".to_string(),
                "Type 'help' for available commands".to_string(),
                "".to_string(),
            ],
            current_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }
    }
}

impl eframe::App for SimpleTerminalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🖥️ Sare Terminal Emulator");
            ui.separator();
            
            ui.group(|ui| {
                ui.label("Output:");
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for line in &self.output_history {
                            ui.label(line);
                        }
                    });
            });
            
            ui.label(format!("📁 Current directory: {}", self.current_dir));
            
            ui.group(|ui| {
                ui.label("💻 Enter command:");
                let response = ui.text_edit_singleline(&mut self.input_text);
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !self.input_text.trim().is_empty() {
                        self.output_history.push(format!("$ {}", self.input_text));
                        self.input_text.clear();
                    }
                }
            });
            
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("💕 Built with love by Yuriko and KleaSCM");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Press Enter to execute commands");
                });
            });
        });
    }
}
