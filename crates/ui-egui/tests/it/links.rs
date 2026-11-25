//! Links: the Help menu and About dialog point to where PdfCraft comes from. This edition carries
//! no ArtCraft mark (its name or logos), and neither the home screen nor the title bar promotes a
//! community.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::links;
use pdfcraft_ui_egui::PdfCraftApp;

fn harness(setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |cc| {
        crate::support::install_runtime(&cc.egui_ctx);
        {
            let mut app = crate::control::new_app();
            app.set_option(&egui::Context::default(), "language", "en").unwrap();
            setup(&mut app);
            app
        }
    });
    h.run_steps(4);
    h
}

#[test]
fn neither_home_nor_the_title_bar_promotes_the_community() {
    let h = harness(|_| {});
    for label in ["Join the ArtCraft community", "Join our Discord", "Discord"] {
        assert!(h.query_by_label(label).is_none(), "{label}");
    }
}

#[test]
fn the_help_menu_has_no_discord_or_artcraft_website_entry() {
    for id in ["help.discord", "help.website"] {
        assert!(pdfcraft_engine::commands::command(id).is_none(), "{id}");
    }
    assert!(links::LINKS.iter().all(|l| !l.label.contains("ArtCraft") && !l.url.contains("discord")));
}

#[test]
fn about_dialog_carries_no_artcraft_mark() {
    let h = harness(|app| app.set_option(&egui::Context::default(), "dialog", "about").unwrap());
    assert_eq!(h.query_all_by_label("ArtCraft").count(), 0, "no ArtCraft logo (it would carry that alt text)");
    assert!(h.query_by_label("Join our Discord").is_none());
}

/// Runs frames until the node labelled `label` exists and has stopped moving, then returns it. A table row is named by
/// its first cell, so a name may be the row's and its cell's: the first (outer) node is the one returned.
///
/// The About dialog is a modal centred on its own size, and switching tabs changes that size, so
/// for a few frames afterwards the modal grows and recentres and every control in it moves. A
/// fixed number of frames is a guess at how long that takes: it was too few after the 0.5.0
/// contributors refresh (78 names instead of 5) on some machines, the click on Table landed where
/// the button had been, and the table never opened (#671).
fn settled<'a>(h: &'a mut Harness<'static, PdfCraftApp>, label: &'a str) -> egui_kittest::Node<'a> {
    let mut last = None;
    for _ in 0..60 {
        h.run_steps(1);
        let now = h.query_all_by_label(label).next().map(|n| n.rect());
        if now.is_some() && now == last {
            return h.query_all_by_label(label).next().expect("the node was just found");
        }
        last = now;
    }
    panic!("{label:?} never appeared or never stopped moving");
}

#[test]
fn about_dialog_has_contributors_and_models_tabs() {
    let mut h = harness(|app| app.set_option(&egui::Context::default(), "dialog", "about").unwrap());
    settled(&mut h, "Contributors").click();
    // The owner is always in the compiled-in credits (contributors/contributors.json), shown by username.
    settled(&mut h, "@echelon");
    settled(&mut h, "Table").click();
    settled(&mut h, "PRs");
    settled(&mut h, "Display name").click();
    settled(&mut h, "Brandon Thomas");
    settled(&mut h, "Models").click();
    h.run_steps(4);
    assert!(h.query_all_by_label("Anthropic").count() >= 1);
}
