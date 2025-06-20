#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use crate::command::ExecutableWithError;
use detect_desktop_environment::DesktopEnvironment;
use regex::Regex;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

mod command;

pub trait SetKeymap {
    fn set_keymap(&self, map: Option<String>, variant: Option<String>) -> Result<(), String>;
}

impl SetKeymap for DesktopEnvironment {
    fn set_keymap(&self, map: Option<String>, variant: Option<String>) -> Result<(), String> {
        match self {
            DesktopEnvironment::Cinnamon => set_keymap_cinnamon(map, variant),
            DesktopEnvironment::Cosmic => todo!(),
            DesktopEnvironment::CosmicEpoch => set_keymap_cosmic_epoch(map, variant),
            DesktopEnvironment::Dde => todo!(),
            DesktopEnvironment::Ede => todo!(),
            DesktopEnvironment::Endless => todo!(),
            DesktopEnvironment::Enlightenment => todo!(),
            DesktopEnvironment::Gnome => set_keymap_gnome(map, variant),
            DesktopEnvironment::Hyprland => set_keymap_hyprland(map, variant),
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
            .execute_with_err()?;
    }

    if let Some(variant) = variant {
        Command::new("kwriteconfig6")
            .arg("--file").arg("kxkbrc")
            .arg("--group").arg("Layout")
            .arg("--key").arg("VariantList")
            .arg(variant)
            .execute_with_err()?;
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
            .execute_with_err()
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
        .output()
        .map_err(|e| format!("Error: Failed to execute gsettings: {e:?}"))?;
    let formats = String::from_utf8(formats.stdout).expect("gsettings returns utf8");
    let formats_list = parse_gnome_output(&formats);

    let index = formats_list.iter().position(|e| e.0 == layout && e.1 == variant);
    let index = index.unwrap_or_else(|| {
        let variant = variant.map(|v| format!("+{v}")).unwrap_or_default();
        let formats = if formats.is_empty() {
            format!("[('xkb', '{layout}{variant}')]")
        } else {
            formats.replace(']', &format!(", ('xkb', '{layout}{variant}')]"))
        };
        //println!("gsettings set org.gnome.desktop.input-sources sources {formats}");
        Command::new("gsettings")
            .arg("set")
            .arg("org.gnome.desktop.input-sources")
            .arg("sources").arg(formats)
            .execute_with_err().unwrap();// TODO: propagate error properly
        formats_list.len()
    });

    Command::new("gsettings")
        .arg("set")
        .arg("org.gnome.desktop.input-sources")
        .arg("current").arg(index.to_string())
        .execute_with_err()?;

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
        .execute_with_err()?;

    if let Some(layout) = layout {
        Command::new("swaymsg")
            .arg("input").arg("type:keyboard")
            .arg("xkb_layout").arg(layout)
            .execute_with_err()?;
    }

    if let Some(variant) = variant {
        Command::new("swaymsg")
            .arg("input").arg("type:keyboard")
            .arg("xkb_variant").arg(variant)
            .execute_with_err()?;
    }
    
    Ok(())
}

#[allow(clippy::format_push_string)]
fn set_keymap_hyprland(layout: Option<String>, variant: Option<String>) -> Result<(), String> {
    // We can parse the current configuration from: `hyprctl getoption input:kb_layout`
    // 
    // I couldn't find a way to ensure we get the correct hyprland configuration file, in case it's
    // not at .config/hypr/hyprland.conf. We need to parse that file and any file it includes until
    // we find a non commented-out `kb_layout`/`kb-variant`. We need to modify that if it exists or
    // otherwise create our own file with the config and include that.
    //
    // What also "works" is just adding our own `input` section to the bottom of the file. Since its
    // easier and less likely to fail horribly when the file format changes, that's what we will do
    // instead.
    //
    // In most configs the configurations the config change will auto-reload, but since this _can_
    // be disabled we still do that ourselves.
    
    let mut config = String::new();
    if let Some(layout) = layout {
        config.push_str(&format!("input:kb_layout = {layout}\n"));
    }
    if let Some(variant) = variant {
        config.push_str(&format!("input:kb_variant = {variant}\n"));
    }
    if config.is_empty() {
        return Ok(())
    }


    let config_dir = env::var("XDG_CONFIG_HOME")
        .unwrap_or("~/.config".to_string());
    let config_dir = PathBuf::from(config_dir);
    let config_file = config_dir.join("hypr").join("hyprland.conf");
    let mut config_file = OpenOptions::new().append(true).open(&config_file)
        .map_err(|_| format!("Error: Failed to open config file at: {}", config_file.display()))?;
    
    config_file.write(format!("\n# --- Begin setwlkeymap generated ---\n\
{config}\
# --- End setwlkeymap generated ---\n").as_bytes())
        .map_err(|e| format!("Error: Failed to write config file: {e}"))?;
    
    
    Command::new("hyprctl")
        .arg("reload")
        .execute_with_err()?;
    Ok(())
}


#[cfg(not(feature = "cosmic"))]
fn set_keymap_cosmic_epoch(_layout: Option<String>, _variant: Option<String>) -> Result<(), String> {
    Err("COSMIC epoch support was disabled during build.".to_string())
}

#[cfg(feature = "cosmic")]
fn set_keymap_cosmic_epoch(layout: Option<String>, variant: Option<String>) -> Result<(), String> {
    use cosmic_config::{ConfigGet, ConfigSet};

    const COSMIC_COMP_CONFIG: &str = "com.system76.CosmicComp";
    const COSMIC_COMP_CONFIG_VERSION: u64 = 1;

    // Do it the same way their settings app does it:
    // https://github.com/pop-os/cosmic-settings/blob/master/cosmic-settings/src/pages/input/keyboard/mod.rs

    let conf = cosmic_config::Config::new(COSMIC_COMP_CONFIG, COSMIC_COMP_CONFIG_VERSION)
        .map_err(|e| format!("Error: Failed to load cosmic config: {e}"))?;
    let mut xkb_conf: cosmic_comp_config::XkbConfig = conf.get("xkb_config")
        .map_err(|e| format!("Error: Failed to set xkb_config: {e}"))?;
    if let Some(layout) = layout {
        xkb_conf.layout = layout;
    }
    if let Some(variant) = variant {
        xkb_conf.variant = variant;
    }
    conf.set("xkb_config", xkb_conf)
        .map_err(|e| format!("Error: Failed to set xkb_config: {e}"))?;

    Ok(())
}


fn parse_gnome_output(formats: &str) -> Vec<(String, Option<String>)> {
    // Regex to match ('xkb', 'de+us') entries, capturing the de+us part in 2 groups
    let formats_regex = Regex::new(r"\('xkb', '(\w*)(\+\w*)?'\)").unwrap();

    let formats = formats_regex.captures_iter(formats);

    formats
        .map(|c| (
            c.get(1).unwrap().as_str().to_string(),
            c.get(2).map(|c| c.as_str()[1..].to_string()),
        ))
        .collect::<Vec<(String, Option<String>)>>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_gnome_output_correctly() {
        let result = super::parse_gnome_output("[('xkb', 'us'), ('xkb', 'ca+eng')]");

        assert_eq!(result.len(), 2);
        assert_eq!(result.first().unwrap().0.as_str(), "us");
        assert_eq!(result.first().unwrap().1, None);
        assert_eq!(result.get(1).unwrap().0.as_str(), "ca");
        let variant = result.get(1).unwrap().1.clone();
        assert!(variant.is_some());
        assert_eq!(variant.unwrap().as_str(), "eng");
    }
}