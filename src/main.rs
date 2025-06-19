#![warn(clippy::pedantic)]

//! Set Wayland Keyboard Map - A unified interface for keyboard layout in Wayland
//! 
//! This is the binary entry point for the setwlkbmap utility.

use clap::Parser;
use detect_desktop_environment::DesktopEnvironment;
use setwlkbmap::SetKeymap;

/// Command line arguments for setwlkbmap
#[derive(Parser, Debug)]
#[clap(
    version = "0.1.0",
    about = "Set Wayland Keyboard Map - A unified interface for setting keyboard layout in Wayland compositors",
    long_about = "This CLI tool aims to provide a consistent way to set keyboard layouts,\nvariants, and XKB options across different Wayland compositors."
)]
struct Args {
    /// Set the primary keyboard layout (e.g., 'us', 'de', 'fr')
    layout: Option<String>,

    /// Set the keyboard variant for the specified layout (e.g., 'altgr-intl', 'dvorak', 'us').
    /// This combines with the layout (e.g., 'de', 'us' for 'de(us)').
    variant: Option<String>,

    /// Detect the current Wayland compositor/desktop environment and exit.
    #[clap(short, long)]
    detect: bool,
}

fn main() {
    let args = Args::parse();

    let Some(de) = DesktopEnvironment::detect() else {
        eprintln!("Failed to detect desktop environment");
        return;
    };

    if args.detect {
        println!("Detected desktop environment: {de:?}");
        return;
    }

    if args.layout.is_none() && args.variant.is_none() {
        eprintln!("Error: Please provide a layout or variant, or use --detect (See --help).");
        return;
    }

    
    match de.set_keymap(args.layout, args.variant) {
        Ok(()) => println!("Keymap set"),
        Err(err) => eprintln!("Error: {err}"),
    }
}
