extern crate game_engine;
use game_engine::{
    components::Component,
    gui::{Button, Gui, HorizontalLayout, Label, TextField, VerticalLayout},
    network::{NetworkClient, NetworkServer, Transport},
    Application,
};

struct NetworkManagerHUD {
    manager: NetworkManager,
    offset_x: i32,
    offset_y: i32,
}

impl NetworkManagerHUD {
    fn new(manager: NetworkManager) -> Self {
        NetworkManagerHUD {
            manager,
            offset_x: 10,
            offset_y: 40,
        }
    }

    fn on_gui(&self) {
        let mut gui = Gui::new(Rect::new(
            self.offset_x,
            self.offset_y,
            300,
            9999,
        ));

        if !NetworkClient::is_connected() && !NetworkServer::is_active() {
            self.start_buttons(&mut gui);
        } else {
            self.status_labels(&mut gui);
        }

        if NetworkClient::is_connected() && !NetworkClient::is_ready() {
            if gui.button("Client Ready").clicked() {
                NetworkClient::ready();
                if NetworkClient::local_player().is_none() {
                    NetworkClient::add_player();
                }
            }
        }

        self.stop_buttons(&mut gui);
    }

    fn start_buttons(&self, gui: &mut Gui) {
        if !NetworkClient::is_active() {
            if cfg!(target_arch = "wasm32") {
                if gui.button("Single Player").clicked() {
                    NetworkServer::set_dont_listen(true);
                    self.manager.start_host();
                }
            } else {
                if gui.button("Host (Server + Client)").clicked() {
                    self.manager.start_host();
                }
            }

            gui.begin_horizontal();
            if gui.button("Client").clicked() {
                self.manager.start_client();
            }

            self.manager.network_address = gui.text_field(self.manager.network_address).content();
            if let Some(port_transport) = Transport::active().downcast_ref::<PortTransport>() {
                if let Ok(port) = gui.text_field(&port_transport.port().to_string()).content().parse::<u16>() {
                    port_transport.set_port(port);
                }
            }
            gui.end_horizontal();

            if cfg!(target_arch = "wasm32") {
                gui.label("( WebGL cannot be server )");
            } else {
                if gui.button("Server Only").clicked() {
                    self.manager.start_server();
                }
            }
        } else {
            gui.label(&format!("Connecting to {}..", self.manager.network_address));
            if gui.button("Cancel Connection Attempt").clicked() {
                self.manager.stop_client();
            }
        }
    }

    fn status_labels(&self, gui: &mut Gui) {
        if NetworkServer::is_active() && NetworkClient::is_active() {
            gui.label("<b>Host</b>: running via Transport.active()");
        } else if NetworkServer::is_active() {
            gui.label("<b>Server</b>: running via Transport.active()");
        } else if NetworkClient::is_connected() {
            gui.label(&format!("<b>Client</b>: connected to {} via Transport.active()", self.manager.network_address));
        }
    }

    fn stop_buttons(&self, gui: &mut Gui) {
        if NetworkServer::is_active() && NetworkClient::is_connected() {
            gui.begin_horizontal();
            if cfg!(target_arch = "wasm32") {
                if gui.button("Stop Single Player").clicked() {
                    self.manager.stop_host();
                }
            } else {
                if gui.button("Stop Host").clicked() {
                    self.manager.stop_host();
                }
                if gui.button("Stop Client").clicked() {
                    self.manager.stop_client();
                }
            }
            gui.end_horizontal();
        } else if NetworkClient::is_connected() {
            if gui.button("Stop Client").clicked() {
                self.manager.stop_client();
            }
        } else if NetworkServer::is_active() {
            if gui.button("Stop Server").clicked() {
                self.manager.stop_server();
            }
        }
    }
}

fn main() {
    let app = Application::new();
    let network_manager = NetworkManager::new();
    let hud = NetworkManagerHUD::new(network_manager);

    app.run(move |frame| {
        hud.on_gui();
    });
}
