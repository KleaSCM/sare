use eframe::egui;

fn main() -> Result<(), eframe::Error> {
	println!("Starting GUI test...");
	
	let options = eframe::NativeOptions::default();
	eframe::run_native(
		"Simple Test",
		options,
		Box::new(|_cc| Box::new(SimpleApp::default())),
	)
}

struct SimpleApp;

impl Default for SimpleApp {
	fn default() -> Self {
		println!("Creating SimpleApp");
		Self
	}
}

impl eframe::App for SimpleApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		println!("Updating GUI...");
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("Hello from Sare!");
			ui.label("If you can see this, the GUI is working!");
			if ui.button("Click me!").clicked() {
				println!("Button clicked!");
			}
		});
	}
} 