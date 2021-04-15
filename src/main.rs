use imgui::*;
use std::{fs::copy, path::PathBuf};

mod support;

/// Get the FTL save directory for all platforms
fn get_save_directory() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        dirs::data_dir().unwrap_or_default().join("FasterThanLight")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir().unwrap_or_default().join("FasterThanLight")
    }
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().unwrap_or_default().join("FasterThanLight")
    }
}

/// Get the FTL save file
fn get_save_file() -> PathBuf {
    get_save_directory().join("continue.sav")
}

fn get_available_saves() -> Vec<PathBuf> {
    std::fs::read_dir(get_save_directory())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|f| f.path().to_path_buf())
        .filter(|f| f.extension() == Some(std::ffi::OsStr::new("msav")))
        .collect()
}

/// Creates backup copy of continue.sav
fn backup() {
    let _ = copy(get_save_file(), get_save_directory().join("backup.sav"));
}


fn main() {
    dbg!(get_save_directory());
    let mut dimensions = (500,700);
    let mut system = support::init(file!(), dimensions);

    let mut savegames = get_available_saves();
    let mut save_name: ImString = im_str!("{}", "unnamed");

    system.imgui.style_mut().window_border_size = 0.0;
    
    system.main_loop(move |_, ui| {

        let s = ui.io().display_size;
        dimensions.0 = s[0] as u32;
        dimensions.1 = s[1] as u32;
        Window::new(im_str!("Savegames"))
        .position([0.0, 0.0], Condition::FirstUseEver)
        .movable(false)
        .no_decoration()
        .size([dimensions.0 as f32, dimensions.1 as f32], Condition::Always)
            .build(ui, || {
                ui.text(im_str!("Available savegames"));
                ui.separator();

                for savegame in &savegames {

                    if ui.button(
                        &im_str!(
                            "Restore {}",
                            savegame.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        [0., 0.],
                    ) {
                        backup();
                        let _ = copy(savegame, get_save_file());
                    }
                }

                ui.separator();

                ui.input_text(im_str!("Save name"), &mut save_name).build();

                if ui.button(&im_str!("Save {}", save_name), [0., 0.]) {
                    match copy(
                        get_save_file(),
                        get_save_directory()
                            .join(save_name.to_string())
                            .with_extension("msav"),
                    ) {
                        Ok(_) => savegames = get_available_saves(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            });

    });
}
