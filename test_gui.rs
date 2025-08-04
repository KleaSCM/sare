use eframe::egui;

fn main() -> Result<(), eframe::Error> {
	println!("Starting minimal GUI test...");
	
	let options = eframe::NativeOptions::default();
	
	eframe::run_native(
		"GUI Test",
		options,
		Box::new(|_cc| Box::new(TestApp::default())),
	)
}

struct TestApp;

impl Default for TestApp {
	fn default() -> Self {
		println!("Creating TestApp");
		Self
	}
}

impl eframe::App for TestApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		println!("Updating GUI...");
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("🌸 GUI Test Window 🌸");
			ui.label("If you can see this, GUI is working!");
			if ui.button("Click me!").clicked() {
				println!("Button clicked!");
			}
		});
	}
} 