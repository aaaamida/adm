use std::{collections::HashSet, sync::mpsc::{Receiver, Sender}};

use eframe::egui;
use egui::{Color32, Stroke};

use crate::rpc::{worker::{RpcCommand, RpcResponse}, Download};

#[allow(unused)]
#[derive(PartialEq)]
enum Filter {
        All,
        Active,
        Waiting,
        Stopped,
        // Error,
}

#[derive(PartialEq)]
enum OnFinish {
        Nothing,
        Exit,
        Suspend,
        Shutdown
}

#[allow(unused)]
pub struct App {
        // channels
        cmd: Sender<RpcCommand>,
        res: Receiver<RpcResponse>,

        // download statuses
        active: Vec<Download>,
        waiting: Vec<Download>,
        stopped: Vec<Download>,
        error: Option<String>,

        // gui options
        filter: Filter,
        on_finish: OnFinish,
        show_add_dialog: bool,
        show_delete_dialog: bool,
        uri_input: String,
        last_refresh: std::time::Instant,
        selected: HashSet<String>
}

impl App {
        pub fn new(tx: Sender<RpcCommand>, rx: Receiver<RpcResponse>) -> Self {
                let app = App {
                        cmd: tx,
                        res: rx,

                        active: vec![],
                        waiting: vec![],
                        stopped: vec![],
                        error: None,

                        filter: Filter::Active,
                        on_finish: OnFinish::Nothing,
                        show_add_dialog: false,
                        show_delete_dialog: false,
                        uri_input: "".into(),
                        last_refresh: std::time::Instant::now(),
                        selected: HashSet::new(),
                };

                app.cmd.send(RpcCommand::TellActive).ok();
                app.cmd.send(RpcCommand::TellWaiting).ok();
                app.cmd.send(RpcCommand::TellStopped).ok();

                app
        }

        // BUG:: set the ui to refresh continously instead of refreshing only when the cursor is
        //       moving
        fn poll_response(&mut self) {
                while let Ok(response) = self.res.try_recv() {
                        println!("[{}] got response {:#?}", chrono::Local::now().format("%Y-%m-%d | %H:%M:%S"), response);
                        match response {
                                RpcResponse::ActiveDownloads(dl) => self.active = dl,
                                RpcResponse::WaitingDownloads(dl) => self.waiting = dl,
                                RpcResponse::StoppedDownloads(dl) => self.stopped = dl,
                                RpcResponse::Error(e) => self.error = Some(e),
                                RpcResponse::Gid(_) => {
                                        self.cmd.send(RpcCommand::TellActive).ok();
                                        self.cmd.send(RpcCommand::TellWaiting).ok();
                                }
                                RpcResponse::OK => (),
                                _ => {}
                        }
                }
        }

        // ui stuff here
        // fn title_bar(&mut self, ui: &mut egui::Ui) {
        //         ui.with_layout( egui::Layout::top_down(egui::Align::Center), |ui| ui.heading("ADM"));
        // }

        fn menu_bar(&mut self, ui: &mut egui::Ui) {
                ui.menu_button("Files", |ui| {
                        if ui.button("Add Torrent File...").clicked() {}
                        if ui.button("Add Torrent Link...").clicked() {}
                        if ui.button("Quit ADM").clicked() {
                                std::process::exit(0);
                        }
                });
                ui.menu_button("Edit", |ui| {
                        if ui.button("Start Downloads").clicked() {}
                        if ui.button("Stop Downloads").clicked() {}
                });
                ui.menu_button("Tools", |ui| {
                        if ui.button("Preferences").clicked() {}
                        ui.menu_button("On Finish...", |ui| {
                                if ui.selectable_label(self.on_finish == OnFinish::Nothing,"Do nothing").enabled() {
                                        self.on_finish = OnFinish::Nothing
                                }
                                if ui.selectable_label(self.on_finish == OnFinish::Exit,"Exit program").enabled() {
                                        self.on_finish = OnFinish::Exit
                                }
                                if ui.selectable_label(self.on_finish == OnFinish::Suspend,"Suspend the system").enabled() {
                                        self.on_finish = OnFinish::Suspend
                                }
                                if ui.selectable_label(self.on_finish == OnFinish::Shutdown,"Shut down the system").enabled() {
                                        self.on_finish = OnFinish::Shutdown
                                }
                        })
                });
        }

        fn toolbar(&mut self, ui: &mut egui::Ui) {
                use egui::Separator;

                ui.set_min_height(60.0);

                ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(20.0, ui.spacing().item_spacing.y);

                        if ui.button("Add...").clicked() {
                                self.show_add_dialog = true;
                        }
                        ui.add(Separator::default().grow(5.0));

                        if ui.button("Pause").clicked() {
                                for gid in &self.selected {
                                        self.cmd.send(RpcCommand::PauseDownload(gid.clone())).ok();
                                }
                        }
                        ui.add(Separator::default().grow(5.0));

                        if ui.button("Resume").clicked() {
                                for gid in &self.selected {
                                        self.cmd.send(RpcCommand::UnpauseDownload(gid.clone())).ok();
                                }
                        }
                        ui.add(Separator::default().grow(5.0));

                        if ui.button("Delete").clicked() {
                                self.show_delete_dialog = true;
                        }
                        ui.add(Separator::default().grow(5.0));
                });
        }

        fn side_panel(&mut self, ui: &mut egui::Ui) {
                ui.spacing_mut().item_spacing = egui::vec2(ui.spacing().item_spacing.x, 4.0);

                ui.add_space(4.0);
                ui.label(egui::RichText::new("Filter").size(14.0));

                if ui.selectable_label(self.filter == Filter::All, "All").clicked() {
                        self.filter = Filter::All
                }

                if ui.selectable_label(self.filter == Filter::Active, "Active").clicked() {
                        self.filter = Filter::Active
                }

                if ui.selectable_label(self.filter == Filter::Waiting, "Waiting").clicked() {
                        self.filter = Filter::Waiting
                }

                if ui.selectable_label(self.filter == Filter::Stopped, "Stopped").clicked() {
                        self.filter = Filter::Stopped
                }
        }

        // BUG: as follows:
        //      - when running w empty downloads in memory,
        //        adding a download for the first time causes
        //        the item to appear in Waiting filter instead
        //        of appearing for a brief moment in Waiting
        //        then moving to Active, which should be the case.
        //      - downloads only display in the correct filter
        //        only when at least one download exists in memory.
        //      - the only way to fix the incorrect filter display
        //        is to restart the program.
        fn central_panel(&mut self, ui: &mut egui::Ui) {
                use egui_extras::Column;

                let cur_filter = match self.filter {
                        Filter::All     => &self.active, // TODO: impl "all downloads" filter later
                        Filter::Active  => &self.active,
                        Filter::Waiting => &self.waiting,
                        Filter::Stopped => &self.stopped,
                };

                #[allow(unused_mut)]
                egui::ScrollArea::vertical().show(ui, |ui| egui_extras::TableBuilder::new(ui)
                        .column(Column::exact(10.0))
                        .column(Column::auto().resizable(true))
                        .column(Column::auto().resizable(true))
                        .column(Column::auto().resizable(true))
                        .column(Column::auto().resizable(true))
                        .column(Column::remainder())
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .header(20.0, |mut header| {
                                header.col(|_| {});
                                header.col(|ui| {
                                        ui.set_min_width(180.0);
                                        ui.label("File Name(s)");
                                });
                                header.col(|ui| {
                                        ui.set_min_width(100.0);
                                        ui.label("Size");
                                });
                                header.col(|ui| {
                                        ui.set_min_width(100.0);
                                        ui.label("Status");
                                });
                                header.col(|ui| {
                                        ui.set_min_width(150.0);
                                        ui.label("Progress");
                                });
                                header.col(|ui| {
                                        if self.filter == Filter::Active {
                                                ui.set_min_width(100.0);
                                                ui.label("Down Speed");
                                        }
                                });
                        })
                        .body(|mut body| {
                                body.rows(20.0, cur_filter.len(), |mut row| {
                                        use byte_unit::{Byte, UnitType};

                                        let row_index = row.index();
                                        let item = cur_filter.get(row_index).unwrap();

                                        let path = item.files.first()
                                                .map(|f| f.path.as_str())
                                                .unwrap_or("unknown path.");
                                        let path = std::path::Path::new(path).file_name().unwrap().to_str().unwrap();
                                        let size = item.files.first()
                                                .map(|f| f.length.as_str())
                                                .unwrap_or("unknown size.");
                                        let status = &item.status;
                                        // TODO: convert from raw bytes to human readable
                                        let total_len = &item.total_length.clone().unwrap_or("0".into());
                                        let compl_len = &item.completed_length.clone().unwrap_or("0".into());
                                        let dl_speed = &item.download_speed.clone().unwrap_or("0".into());

                                        let f_size = Byte::from_u64(size.parse().unwrap())
                                                .get_appropriate_unit(UnitType::Decimal);
                                        let t_len = Byte::from_u64(total_len.parse().unwrap())
                                                .get_appropriate_unit(UnitType::Decimal);
                                        let c_len = Byte::from_u64(compl_len.parse().unwrap())
                                                .get_appropriate_unit(UnitType::Decimal);
                                        let dl_speed = Byte::from_u64(dl_speed.parse().unwrap())
                                                .get_appropriate_unit(UnitType::Decimal);

                                        row.col(|ui| {
                                                let mut checked = self.selected.contains(&item.gid);
                                                if ui.checkbox(&mut checked, "").clicked() {
                                                        if checked {
                                                                self.selected.insert(item.gid.clone());
                                                        } else {
                                                                self.selected.remove(&item.gid);
                                                        }
                                                }
                                        });
                                        row.col(|ui| {
                                                ui.label(path);
                                        });
                                        row.col(|ui| {
                                                ui.label(format!("{f_size:.2}"));
                                        });
                                        row.col(|ui| {
                                                ui.label(status);
                                        });
                                        row.col(|ui| {
                                                ui.label(format!("{c_len:.2} / {t_len:.2}"));
                                        });
                                        row.col(|ui| {
                                                if self.filter == Filter::Active {
                                                        ui.label(format!("{dl_speed:.2}"));
                                                }
                                        });
                                });
                        })
                );
        }

        fn add_download_dialog(&mut self, ui: &mut egui::Ui) {
                let title = egui::RichText::new("Add Download")
                        .size(14.0)
                        .color(Color32::WHITE);

                egui::Window::new(title)
                        .fixed_size([300.0, 150.0])
                        .collapsible(false)
                        .resizable(false)
                        // .default_pos([ui.min_size().x / 2.0, ui.min_size().y / 2.0])
                        .show(ui.ctx(), |ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);
                                ui.label("URL");
                                ui.text_edit_singleline(&mut self.uri_input);

                                ui.horizontal(|ui| {
                                        if ui.button("Add").clicked() {
                                                self.cmd.send(RpcCommand::AddUri(self.uri_input.clone())).ok();
                                                self.uri_input.clear();
                                                self.show_add_dialog = false;
                                        }
                                        if ui.button("Cancel").clicked() {
                                                self.show_add_dialog = false;
                                        }
                                })
                        });
        }

        fn delete_confirm_dialog(&mut self, ui: &mut egui::Ui) {
                let title = egui::RichText::new("Delete Confirmation")
                        .size(14.0)
                        .color(Color32::WHITE);

                egui::Window::new(title)
                        .fixed_size([150.0, 100.0])
                        .resizable(false)
                        .collapsible(false)
                        .fixed_pos(ui.globally_used_rect().center())
                        .show(ui.ctx(), |ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);
                                ui.label("Are you sure you want to delete selected items?");

                                ui.horizontal(|ui| {
                                        if ui.button("Confirm").clicked() {
                                                for gid in &self.selected {
                                                        match self.filter {
                                                                Filter::All     => {
                                                                        self.cmd.send(RpcCommand::RemoveResult(gid.clone())).ok();
                                                                        self.cmd.send(RpcCommand::RemoveDownload(gid.clone())).ok();
                                                                }
                                                                Filter::Stopped => _ = self.cmd.send(RpcCommand::RemoveResult(gid.clone())),
                                                                _               => _ = self.cmd.send(RpcCommand::RemoveDownload(gid.clone())),
                                                        }
                                                }
                                                self.selected.clear();
                                                self.show_delete_dialog = false;
                                                self.cmd.send(RpcCommand::TellActive).ok();
                                                self.cmd.send(RpcCommand::TellWaiting).ok();
                                                self.cmd.send(RpcCommand::TellStopped).ok();
                                        }
                                        if ui.button("Cancel").clicked() {
                                                self.show_delete_dialog = false;
                                        }
                                })
                        });
        }
}

impl eframe::App for App {
        fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
                self.poll_response();

                if self.last_refresh.elapsed().as_secs() >= 1 {
                        self.cmd.send(RpcCommand::TellActive).ok();
                        self.cmd.send(RpcCommand::TellWaiting).ok();
                        self.cmd.send(RpcCommand::TellStopped).ok();
                        self.last_refresh = std::time::Instant::now();
                }

                ui.visuals_mut().selection.bg_fill = Color32::DARK_GRAY;
                ui.visuals_mut().selection.stroke = Stroke::new(8.0, Color32::WHITE);

                // title bar
                // egui::Panel::top("title_bar").resizable(false).show_inside(ui, |ui| self.title_bar(ui));

                // menu buttons
                egui::Panel::top("menu_bar").show_inside(ui, |ui| egui::MenuBar::new().ui(ui, |ui| self.menu_bar(ui)));
                egui::Panel::top("toolbar").show_inside(ui, |ui| egui::MenuBar::new().ui(ui, |ui| self.toolbar(ui)));

                if self.show_add_dialog {
                        self.add_download_dialog(ui);
                }

                if self.show_delete_dialog {
                        self.delete_confirm_dialog(ui);
                }

                // function buttons
                egui::Panel::left("functions")
                        .resizable(true)
                        .min_size(80.0)
                        .max_size(180.0)
                        .default_size(80.0)
                        .show_inside(ui, |ui| self.side_panel(ui));

                // main content
                egui::CentralPanel::default().show_inside(ui, |ui| self.central_panel(ui));
        }
}
