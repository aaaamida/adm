use eframe::egui;

#[derive(Default)]
pub struct App {}

impl App {
        fn title_bar(&mut self, ui: &mut egui::Ui) {
                ui.with_layout( egui::Layout::top_down(egui::Align::Center), |ui| ui.heading("ADM"));
        }

        fn menu_bar(&mut self, ui: &mut egui::Ui) {
                ui.menu_button("Files", |ui| {
                        if ui.button("Add Torrent File...").clicked() {}
                        if ui.button("Add Torrent Link...").clicked() {}
                        if ui.button("Quit ADM").clicked() {}
                });
                ui.menu_button("Edit", |ui| {
                        if ui.button("Start Downloads").clicked() {}
                        if ui.button("Stop Downloads").clicked() {}
                });
                ui.menu_button("Tools", |ui| {
                        if ui.button("Preferences").clicked() {}
                        ui.menu_button("On Finish...", |ui| {
                                if ui.button("Do nothing").enabled() {}
                                if ui.button("Exit program").enabled() {}
                                if ui.button("Suspend the system").enabled() {}
                                if ui.button("Shut down the system").enabled() {}
                        })
                });
        }

        #[allow(unused)]
        fn central_panel(&mut self, ui: &mut egui::Ui) { }
}

impl eframe::App for App {
        fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
                // title bar
                egui::Panel::top("title_bar").resizable(false).show_inside(ui, |ui| self.title_bar(ui));

                // menu buttons
                egui::Panel::top("menu_bar").show_inside(ui, |ui| egui::MenuBar::new().ui(ui, |ui| self.menu_bar(ui)));

                // function buttons
                // egui::Panel::top("functions")
                //         .resizable(false)
                //         .exact_size(60.0)
                //         .show_inside(ui, |ui| {
                //                 if ui.button("Add").clicked() {}
                // });

                // main content
                egui::CentralPanel::default().show_inside(ui, |ui| self.central_panel(ui));
        }

        // #[allow(unused)]
        // fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //         todo!()
        // }
}
