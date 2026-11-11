#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use ui::{show_error_dialog, WindhawkApp};
use windhawk_core::ensure_os_supported;

fn main() {
    // Initialize tracing logger
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Windhawk Windows 10 Notes & Tweaks Exporter");

    // Phase 3 Requirement 2:
    // Runtime guard using RtlGetVersion that refuses to run on any build outside
    // the Windows 10 range (19041-21999) with a clear error dialog and message.
    let os_version = match ensure_os_supported() {
        Ok(v) => v,
        Err(err) => {
            let error_title = "Windhawk — Incompatible Windows Version";
            let error_message = format!(
                "This application is strictly designed for Windows 10 (Builds 19041 to 21999).\n\n\
                 Compatibility check failed:\n{}\n\n\
                 Windows 11 (build >= 22000) is already supported upstream by the official Windhawk release.\n\
                 The application will now exit.",
                err
            );

            error!("{}: {}", error_title, error_message);
            show_error_dialog(error_title, &error_message);
            std::process::exit(1);
        }
    };

    info!(
        "OS Guard passed: Windows 10 Build {} verified. Launching Modern Fancy GUI.",
        os_version.build_number
    );

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([980.0, 660.0])
            .with_min_inner_size([820.0, 520.0])
            .with_title(format!(
                "Windhawk — Windows 10 Notes & Tweaks Exporter (Build {})",
                os_version.build_number
            )),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Windhawk Notes & Tweaks Exporter",
        native_options,
        Box::new(move |_cc| Ok(Box::new(WindhawkApp::new(os_version)))),
    ) {
        error!("Fatal GUI runtime error: {:?}", e);
        show_error_dialog("Windhawk — Runtime Error", &format!("{:?}", e));
        std::process::exit(1);
    }

    info!("Windhawk Windows 10 Notes & Tweaks Exporter closed normally.");
}
