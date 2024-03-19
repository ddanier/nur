use nu_ansi_term::Color;
use std::path::Path;

pub(crate) fn show_nurscripts_hint(project_path: &Path, use_color: bool) {
    // Give some hints if old ".nurscripts" exists
    let old_nur_lib_path = project_path.join(".nurscripts");
    if old_nur_lib_path.exists() && old_nur_lib_path.is_dir() {
        eprintln!(
            "{}WARNING: .nurscripts/ has moved to .nur/scripts/ -> please update your project{}",
            if use_color {
                Color::Red.prefix().to_string()
            } else {
                String::from("")
            },
            if use_color {
                Color::Red.suffix().to_string()
            } else {
                String::from("")
            },
        );
    }
}
