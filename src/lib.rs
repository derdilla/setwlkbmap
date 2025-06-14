use std::process::{Command, Stdio};

use detect_desktop_environment::DesktopEnvironment;

pub trait SetKeymap {
    fn set_keymap(&self, map: Option<String>, variant: Option<String>) -> Result<(), String>;
}

impl SetKeymap for DesktopEnvironment {
    fn set_keymap(&self, map: Option<String>, variant: Option<String>) -> Result<(), String> {
        match self {
            DesktopEnvironment::Cinnamon => todo!(),
            DesktopEnvironment::Cosmic => todo!(),
            DesktopEnvironment::CosmicEpoch => todo!(),
            DesktopEnvironment::Dde => todo!(),
            DesktopEnvironment::Ede => todo!(),
            DesktopEnvironment::Endless => todo!(),
            DesktopEnvironment::Enlightenment => todo!(),
            DesktopEnvironment::Gnome => todo!(),
            DesktopEnvironment::Hyprland => todo!(),
            DesktopEnvironment::Kde => set_keymap_kde(map, variant),
            DesktopEnvironment::Lxde => todo!(),
            DesktopEnvironment::Lxqt => todo!(),
            DesktopEnvironment::MacOs => todo!(),
            DesktopEnvironment::Mate => todo!(),
            DesktopEnvironment::Old => todo!(),
            DesktopEnvironment::Pantheon => todo!(),
            DesktopEnvironment::Razor => todo!(),
            DesktopEnvironment::Rox => todo!(),
            DesktopEnvironment::Sway => todo!(),
            DesktopEnvironment::Tde => todo!(),
            DesktopEnvironment::Unity => todo!(),
            DesktopEnvironment::Windows => todo!(),
            DesktopEnvironment::Xfce => set_keymap_xfce(map, variant),
            de => Err(format!("settings keymap unimplemented for {de:?}")),
        }
    }

    

}

fn set_keymap_kde(layout: Option<String>, variant: Option<String>) -> Result<(), String> {
    // KDEs configuration is stored in ini style files at .config/...rc.
    // kreadconfig and kwriteconfig can be used to interact with these files. Thez are
    // available on all kde based systems and automatically notify kde about the change via dbus.

    // Currently we don't check whether the layout options are valid since custom layouts can be
    // configured and existing ones can be renamed.
    if let Some(layout) = layout {
        Command::new("kwriteconfig6")
            .arg("--file").arg("kxkbrc")
            .arg("--group").arg("Layout")
            .arg("--key").arg("LayoutList")
            .arg(layout)
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status()
            .map_err(|e| format!("Failed to execute kwriteconfig6: {}", e))
            .and_then(|status| {
                if status.success() { Ok(()) } else { Err("kwriteconfig6 failed".to_string()) }
            })?;
    }

    if let Some(variant) = variant {
        Command::new("kwriteconfig6")
            .arg("--file").arg("kxkbrc")
            .arg("--group").arg("Layout")
            .arg("--key").arg("VariantList")
            .arg(variant)
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status()
            .map_err(|e| format!("Failed to execute kwriteconfig6: {}", e))
            .and_then(|status| {
                if status.success() { Ok(()) } else { Err("kwriteconfig6 failed".to_string()) }
            })?;
    }
    
    Ok(())
}

fn set_keymap_xfce(layout: Option<String>, variant: Option<String>) -> Result<(), String> {
    // xfce uses `xfconf-query` to get and manage settings. Keyboard layouts are defined in the
    // keyboard-layout category. We must first ensure overriding of keyboard settings is enabled. 

    fn set_property(property: &str, value: &str) -> Result<(), String> {
        Command::new("xfconf-query")
            .arg("-c").arg("keyboard-layout")
            .arg("-p").arg(property)
            .arg("-s").arg(value)
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status()
            .map_err(|e| format!("Failed to execute xfconf-query: {}", e))
            .and_then(|status| {
                if status.success() { Ok(()) } else { Err("xfconf-query failed".to_string()) }
            })
    }

    set_property("/Default/XkbDisable", "false")?;

    if let Some(layout) = layout {
        set_property("/Default/XkbLayout", &layout)?;
    }

    if let Some(variant) = variant {
        set_property("/Default/XkbVariant", &variant)?;
    }
    
    Ok(())
}

