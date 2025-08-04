use eframe::egui;

fn main() -> Result<(), eframe::Error> {
	println!("Starting simple GUI test...");
	
	let options = eframe::NativeOptions {
		initial_window_size: Some(egui::vec2(400.0, 300.0)),
		..Default::default()
	};
	
	eframe::run_native(
		"Simple Test",
		options,
		Box::new(|_cc| Box::new(MyApp::default())),
	)
}

struct MyApp;

impl Default for MyApp {
	fn default() -> Self {
		println!("Creating MyApp");
		Self
	}
}

impl eframe::App for MyApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		println!("Updating GUI...");
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("Hello from Sare!");
			ui.label("This is a test window.");
			if ui.button("Test Button").clicked() {
				println!("Button clicked!");
			}
		});
	}
} 