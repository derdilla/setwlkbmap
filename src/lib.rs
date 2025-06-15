use std::process::{Command, Stdio};
use regex::Regex;

use detect_desktop_environment::DesktopEnvironment;

pub trait SetKeymap {
    fn set_keymap(&self, map: Option<String>, variant: Option<String>) -> Result<(), String>;
}

impl SetKeymap for DesktopEnvironment {
    fn set_keymap(&self, map: Option<String>, variant: Option<String>) -> Result<(), String> {
        match self {
            DesktopEnvironment::Cinnamon => set_keymap_cinnamon(map, variant),
            DesktopEnvironment::Cosmic => todo!(),
            DesktopEnvironment::CosmicEpoch => todo!(),
            DesktopEnvironment::Dde => todo!(),
            DesktopEnvironment::Ede => todo!(),
            DesktopEnvironment::Endless => todo!(),
            DesktopEnvironment::Enlightenment => todo!(),
            DesktopEnvironment::Gnome => set_keymap_gnome(map, variant),
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
            DesktopEnvironment::Sway => set_keymap_sway(map, variant),
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

fn set_keymap_gnome(layout: Option<String>, variant: Option<String>) -> Result<(), String> {
    // Gnome has input sources, and the index of a selected input source 
    // `gsettings get org.gnome.desktop.input-sources sources`
    // > [('xkb', 'us'), ('xkb', 'ca+eng')]
    //   or nothing if the keymap was never set
    // `gsettings get org.gnome.desktop.input-sources current`
    // > uint32 0
    // Layout and variant are seperated by a + sign, layout is always required
    let Some(layout) = layout else {
        return Err("Setting the gnome keymap requires a layout".to_string());
    };
    
    let formats = Command::new("gsettings")
            .arg("get")
            .arg("org.gnome.desktop.input-sources")
            .arg("sources")
            .stdout(Stdio::piped())
            .output()
            .map_err(|e| format!("Error: Failed to execute gsettings: {e:?}"))?;
    let formats = String::from_utf8(formats.stdout).expect("gsettings returns utf8");
    let formats_list = parse_gnome_output(&formats)?;

    let index = formats_list.iter().position(|e| e.0 == layout && e.1 == variant);
    let index = index.unwrap_or_else(|| {
        let variant = variant.map(|v| format!("+{v}")).unwrap_or(String::new());
        let formats = if formats.is_empty() {
            format!("[('xkb', '{layout}{variant}')]")
        } else {
            formats.replace("]", &format!(", ('xkb', '{layout}{variant}')]"))
        };
        //println!("gsettings set org.gnome.desktop.input-sources sources {formats}");
        Command::new("gsettings")
            .arg("set")
            .arg("org.gnome.desktop.input-sources")
            .arg("sources").arg(formats)
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status()
            .map_err(|e| format!("Failed to execute gsettings: {}", e))
            .and_then(|status| {
                if status.success() { Ok(()) } else { Err("gsettings failed".to_string()) }
            }).unwrap();// TODO: propagate error properly
        formats_list.len()
    });

    Command::new("gsettings")
        .arg("set")
        .arg("org.gnome.desktop.input-sources")
        .arg("current").arg(index.to_string())
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status()
        .map_err(|e| format!("Failed to execute gsettings: {}", e))
        .and_then(|status| {
            if status.success() { Ok(()) } else { Err("gsettings failed".to_string()) }
        })?;

    Ok(())
}

fn set_keymap_cinnamon(_layout: Option<String>, _variant: Option<String>) -> Result<(), String> {
    Err("As of June 2025 cinnamon wayland is still experimental and doesn't yet support switching
 the keyboard layout. You can check if that's still the case by opening the keyboard settings dialog
 and looking for an 'Layout' tab. In case this changed consider opening an issue or submitting a
 patch at https://github.com/derdilla/setwlkbmap.".to_string())
}

fn set_keymap_sway(layout: Option<String>, variant: Option<String>) -> Result<(), String> {
    // Sways configuration is stored at ~/.config/sway/config where something the following could be defined:
    // input * {
    //     xkb_layout "de"
    //     xkb_variant "us"
    // }
    // 
    // The easiest way to configure it during runtime is:
    // swaymsg input type:keyboard xkb_layout "de"
    // swaymsg input type:keyboard xkb_variant "us"
    // This does however do the full check whether that layout actually exists after every command,
    // so we need to first clear the old variant.
    // 
    // It also seems like sway may support org.freedesktop.locale1, but this support is be dependent on 
    // the configuration.

    Command::new("swaymsg")
        .arg("input").arg("type:keyboard")
        .arg("xkb_variant").arg("''")
        .stdout(Stdio::null()).stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to execute swaymsg: {}", e))
        .and_then(|status| {
            if status.success() { Ok(()) } else { Err("swaymsg failed".to_string()) }
        })?;

    if let Some(layout) = layout {
        Command::new("swaymsg")
            .arg("input").arg("type:keyboard")
            .arg("xkb_layout").arg(layout)
            .stdout(Stdio::null()).stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to execute swaymsg: {}", e))
            .and_then(|status| {
                if status.success() { Ok(()) } else { Err("swaymsg failed".to_string()) }
            })?;
    }

    if let Some(variant) = variant {
        Command::new("swaymsg")
            .arg("input").arg("type:keyboard")
            .arg("xkb_variant").arg(variant)
            .stdout(Stdio::null()).stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to execute swaymsg: {}", e))
            .and_then(|status| {
                if status.success() { Ok(()) } else { Err("swaymsg failed".to_string()) }
            })?;
    }
    
    Ok(())
}


fn parse_gnome_output(formats: &str) -> Result<Vec<(String, Option<String>)>, String> {
    // Regex to match ('xkb', 'de+us') entries, capturing the de+us part in 2 groups
    let formats_regex = Regex::new(r"\('xkb', '(\w*)(\+\w*)?'\)").unwrap();

    let formats = formats_regex.captures_iter(&formats);

    let formats = formats
        .map(|c| (
            c.get(1).unwrap().as_str().to_string(),
            c.get(2).map(|c| c.as_str()[1..].to_string()),
        ))
        .collect::<Vec<(String, Option<String>)>>();
    Ok(formats)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_gnome_output_correctly() {
        let result = super::parse_gnome_output("[('xkb', 'us'), ('xkb', 'ca+eng')]");
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap().0.as_str(), "us");
        assert_eq!(result.get(0).unwrap().1, None);
        assert_eq!(result.get(1).unwrap().0.as_str(), "ca");
        let variant = result.get(1).unwrap().1.clone();
        assert!(variant.is_some());
        assert_eq!(variant.unwrap().as_str(), "eng");
    }
}