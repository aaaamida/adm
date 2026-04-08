use eframe::egui;

#[derive(Default)]
struct App {}

// impl App {
//         fn new(cc: &eframe::CreationContext<'_>) -> Self {
//                 Self::default()
//         }
// }

impl eframe::App for App {
        fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.heading("ADM");
                });
        }
}

fn main() {
        let native_opts = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 500.0]),
                ..Default::default()
        };

        _ = eframe::run_native(
                "ADM",
                native_opts,
                Box::new(|_| Ok(Box::<App>::default()))
        );
}
