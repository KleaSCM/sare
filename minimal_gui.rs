fn main() {
	println!("Testing minimal GUI...");
	
	// Check if we can access X11
	unsafe {
		use std::ffi::CString;
		let display_name = CString::new(":0").unwrap();
		let display = libc::XOpenDisplay(display_name.as_ptr());
		if display.is_null() {
			println!("ERROR: Cannot open X11 display");
			return;
		}
		println!("SUCCESS: X11 display opened successfully");
		libc::XCloseDisplay(display);
	}
	
	println!("GUI test completed");
} 