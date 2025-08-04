fn main() {
	println!("Testing window creation...");
	
	// Test if we can access X11 directly
	unsafe {
		use std::ffi::CString;
		let display_name = CString::new(":0").unwrap();
		let display = libc::XOpenDisplay(display_name.as_ptr());
		if display.is_null() {
			println!("ERROR: Cannot open X11 display");
			return;
		}
		println!("SUCCESS: X11 display opened successfully");
		
		// Try to create a simple window
		let root = libc::XDefaultRootWindow(display);
		let window = libc::XCreateSimpleWindow(
			display,
			root,
			100, 100, 400, 300,
			1,
			0x000000, // black border
			0xFFFFFF, // white background
		);
		
		if window == 0 {
			println!("ERROR: Cannot create window");
			libc::XCloseDisplay(display);
			return;
		}
		
		println!("SUCCESS: Window created!");
		libc::XMapWindow(display, window);
		libc::XFlush(display);
		
		// Keep window open for a few seconds
		std::thread::sleep(std::time::Duration::from_secs(3));
		
		libc::XDestroyWindow(display, window);
		libc::XCloseDisplay(display);
		println!("Window test completed");
	}
} 