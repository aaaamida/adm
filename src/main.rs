mod ui;
mod rpc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
        let native_opts = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                        .with_inner_size([800.0, 600.0])
                        .with_min_inner_size([400.0, 300.0])
                        .with_title("ADM")
                        .with_decorations(false)
                        .with_minimize_button(true)
                        .with_maximize_button(true)
                        .with_close_button(true),
                ..Default::default()
        };

        rpc::Rpc::connect("ws://127.0.0.1:6800/jsonrpc").await?;

        eframe::run_native(
                "ADM",
                native_opts,
                Box::new(|_| Ok(Box::<ui::App>::default()))
        )?;

        Ok(())
}
