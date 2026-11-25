//! Help ▸ Check for updates and the check at start, with stand-in release sources (no network).
//! This edition's Check for updates opens its own releases page and asks nobody: upstream's
//! releases have none of its changes.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use craft::host::Host;
use craft::update::stack_dialogs::{UpdateCheckDialog, UpdatePromptDialog};
use craft::update::updater::CheckSource;
use egui_kittest::Harness;
use pdfcraft_ui_egui::PdfCraftApp;
use pdfcraft_ui_egui::settings::import_json;

use crate::control::{info, services_asking};
use crate::support::AppHarness as _;

fn release(tag: &str) -> craft::update::Release {
    craft::update::Release {
        tag: tag.to_string(),
        version: craft::update::parse_version(tag),
        html_url: format!("https://github.com/storytold/pdfcraft/releases/tag/{tag}"),
        ..Default::default()
    }
}

/// `Ok(Some(tag))` is a newer release, `Ok(None)` is up to date.
fn source(answer: Result<Option<&str>, &str>) -> CheckSource {
    let answer = answer.map(|tag| tag.map(release)).map_err(craft::update::CheckError::from);
    Arc::new(move || answer.clone())
}

/// A source that counts the times it is asked and always offers `v99.0.0`.
fn counting_source(calls: &Arc<AtomicUsize>) -> CheckSource {
    let calls = calls.clone();
    Arc::new(move || {
        calls.fetch_add(1, SeqCst);
        Ok(Some(release("v99.0.0")))
    })
}

/// An app on the default host whose update check asks `source`.
fn app_asking(source: CheckSource) -> PdfCraftApp {
    let host = Arc::new(Host::builder(info()).build());
    let mut app = PdfCraftApp::with_host(host, services_asking(Some(source)));
    app.set_option(&egui::Context::default(), "language", "en").unwrap();
    app
}

/// The desktop app's start, with `settings` applied, and the count of what the source was asked.
fn started_with(settings: &'static str) -> (Harness<'static, PdfCraftApp>, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let source = counting_source(&calls);
    let h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |cc| {
        crate::support::install_runtime(&cc.egui_ctx);
        {
            let mut app = app_asking(source);
            app.set_startup_update_check(true);
            app.apply_settings(&import_json(settings).0);
            app
        }
    });
    (h, calls)
}

/// Run frames for `time`, or until the source has been asked. The check at start shows nothing
/// while it runs, so `settle` can't wait for it.
fn run_until_asked(h: &mut Harness<'static, PdfCraftApp>, calls: &AtomicUsize, time: std::time::Duration) {
    let deadline = std::time::Instant::now() + time;
    while calls.load(SeqCst) == 0 && std::time::Instant::now() < deadline {
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Whether the settings the app would save have the check at start on; an app that saved no value
/// has the default, which is on.
fn check_at_start_saved(app: &PdfCraftApp) -> bool {
    app.settings_snapshot().layout.update_check_at_start.unwrap_or(true)
}

#[test]
fn the_preference_keeps_the_check_at_start_from_reaching_the_network() {
    let calls = Arc::new(AtomicUsize::new(0));
    assert!(check_at_start_saved(&app_asking(counting_source(&calls))), "on by default");

    // The same start with the preference left on does ask, so a count of none below means
    // the preference held the check back and not that it hadn't run yet.
    let (mut h, calls) = started_with("{}");
    run_until_asked(&mut h, &calls, std::time::Duration::from_secs(30));
    assert_eq!(calls.load(SeqCst), 1, "on, the app asks at start");

    let (mut h, calls) = started_with(r#"{"update_check_at_start": false}"#);
    run_until_asked(&mut h, &calls, std::time::Duration::from_millis(500));
    assert_eq!(calls.load(SeqCst), 0, "nothing is asked at start");

    let mut restarted = app_asking(source(Ok(None)));
    restarted.apply_settings(&h.state().settings_snapshot());
    assert!(!check_at_start_saved(&restarted), "the choice is kept");
}

#[test]
fn settings_from_the_old_opt_in_leave_the_check_at_start_on() {
    let mut app = app_asking(source(Ok(None)));
    app.apply_settings(&import_json(r#"{"check_updates_at_start": false}"#).0);
    assert!(check_at_start_saved(&app), "a key of the removed option decides nothing");
    assert_ne!(app.settings_snapshot().layout.update_check_at_start, Some(false));
}

/// Whether an update dialog is on the host: the check's result or the prompt of a found release.
fn update_dialog_open(h: &Harness<'static, PdfCraftApp>) -> bool {
    h.state().dialogs.is_open::<UpdateCheckDialog>() || h.state().dialogs.is_open::<UpdatePromptDialog>()
}

#[test]
fn checking_for_updates_asks_no_one_and_offers_no_download() {
    let calls = Arc::new(AtomicUsize::new(0));
    let source = counting_source(&calls);
    let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |cc| {
        crate::support::install_runtime(&cc.egui_ctx);
        app_asking(source)
    });
    h.run_steps(4);
    // The desktop app never turns the check at start on, so nothing was asked by now either.
    assert_eq!(calls.load(SeqCst), 0, "nothing is asked at start");
    assert!(h.execute("help.check_updates"));
    h.run_steps(6);
    assert_eq!(calls.load(SeqCst), 0, "upstream's releases would replace this edition, so the command only opens our page");
    assert!(!update_dialog_open(&h), "no release is offered");
}

