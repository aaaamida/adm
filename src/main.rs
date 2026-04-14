use crate::ui::App;

mod ui;
mod rpc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
        use crate::rpc::worker::*;

        let native_opts = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                        .with_inner_size([800.0, 600.0])
                        .with_min_inner_size([600.0, 300.0])
                        .with_max_inner_size([1200.0, 900.0])
                        .with_title("ADM")
                        .with_decorations(false)
                        .with_minimize_button(true)
                        .with_maximize_button(true)
                        .with_close_button(true),
                ..Default::default()
        };

        // rpc::Rpc::connect("ws://127.0.0.1:6800/jsonrpc").await?;

        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<RpcCommand>();
        let (res_tx, res_rx) = std::sync::mpsc::sync_channel::<RpcResponse>(32);

        spawn_worker(cmd_rx, res_tx);

        eframe::run_native(
                "ADM",
                native_opts,
                Box::new(|_| Ok(Box::new(App::new(cmd_tx, res_rx))))
        )?;

        Ok(())
}
