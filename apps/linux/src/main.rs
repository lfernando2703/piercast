use relm4::prelude::*;
use relm4::adw;
use serde::Deserialize;

const DEFAULT_BASE: &str = "http://127.0.0.1:47923";

#[derive(Debug, Clone, Deserialize)]
struct PiercastAppInfo {
    id: String,
    name: String,
    status: String,
}

struct App {
    apps: Vec<PiercastAppInfo>,
    filter: Filter,
    connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Running,
    Unhealthy,
    Favorites,
}

#[derive(Debug)]
enum Msg {
    Refresh,
    SetFilter(Filter),
    Open(String),
    Stop(String),
    Kill(String),
}

fn data_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("piercast")
}

fn load_token() -> Option<String> {
    let path = data_dir().join("pairing.json");
    let text = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("token")?.as_str().map(|s| s.to_string())
}

fn base_url() -> String {
    std::env::var("PIERCAST_URL").unwrap_or_else(|_| DEFAULT_BASE.to_string())
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("http client")
}

fn ensure_daemon() {
    let url = format!("{}/v1/health", base_url());
    if client().get(&url).send().map(|r| r.status().is_success()).unwrap_or(false) {
        return;
    }
    let _ = std::process::Command::new("piercastd").spawn();
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(250));
        if client().get(&url).send().map(|r| r.status().is_success()).unwrap_or(false) {
            return;
        }
    }
}

fn fetch_apps() -> (bool, Vec<PiercastAppInfo>) {
    ensure_daemon();
    let mut req = client().get(format!("{}/v1/apps", base_url()));
    if let Some(token) = load_token() {
        req = req.bearer_auth(token);
    }
    match req.send().and_then(|r| r.error_for_status()).and_then(|r| r.json()) {
        Ok(apps) => (true, apps),
        Err(_) => (false, vec![]),
    }
}

fn post_action(path: &str) {
    let mut req = client().post(format!("{}/{}", base_url(), path));
    if let Some(token) = load_token() {
        req = req.bearer_auth(token);
    }
    let _ = req.send();
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("Piercast"),
            set_default_width: 960,
            set_default_height: 640,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_width_request: 200,
                    set_margin_all: 8,
                    set_spacing: 4,

                    gtk::Label {
                        set_label: "Library",
                        set_halign: gtk::Align::Start,
                        add_css_class: "title-4",
                    },
                    gtk::Button {
                        set_label: "All",
                        connect_clicked => Msg::SetFilter(Filter::All),
                    },
                    gtk::Button {
                        set_label: "Running",
                        connect_clicked => Msg::SetFilter(Filter::Running),
                    },
                    gtk::Button {
                        set_label: "Unhealthy",
                        connect_clicked => Msg::SetFilter(Filter::Unhealthy),
                    },
                    gtk::Button {
                        set_label: "Favorites",
                        connect_clicked => Msg::SetFilter(Filter::Favorites),
                    },
                    gtk::Button {
                        set_label: "Refresh",
                        connect_clicked => Msg::Refresh,
                    },
                },

                gtk::Separator {},

                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,

                    gtk::ListBox {
                        set_selection_mode: gtk::SelectionMode::None,
                        set_margin_all: 12,
                        set_spacing: 8,
                        #[watch]
                        set_visible: !model.apps.is_empty(),

                        append = &gtk::Label {
                            #[watch]
                            set_label: &format!(
                                "{} — {}",
                                if model.connected { "Connected" } else { "Disconnected" },
                                match model.filter {
                                    Filter::All => "All",
                                    Filter::Running => "Running",
                                    Filter::Unhealthy => "Unhealthy",
                                    Filter::Favorites => "Favorites",
                                }
                            ),
                            set_halign: gtk::Align::Start,
                            add_css_class: "title-2",
                            set_margin_bottom: 8,
                        },
                    },
                },
            },
        }
    }

    fn init(_init: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let (connected, apps) = fetch_apps();
        let model = App {
            apps,
            filter: Filter::All,
            connected,
        };
        let widgets = view_output!();
        // Populate list after first paint via refresh
        sender.input(Msg::Refresh);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Refresh => {
                let (connected, apps) = fetch_apps();
                self.connected = connected;
                self.apps = apps
                    .into_iter()
                    .filter(|a| match self.filter {
                        Filter::All => true,
                        Filter::Running => a.status.eq_ignore_ascii_case("running"),
                        Filter::Unhealthy => a.status.eq_ignore_ascii_case("unhealthy"),
                        Filter::Favorites => false,
                    })
                    .collect();
            }
            Msg::SetFilter(f) => {
                self.filter = f;
                let (connected, apps) = fetch_apps();
                self.connected = connected;
                self.apps = apps
                    .into_iter()
                    .filter(|a| match self.filter {
                        Filter::All => true,
                        Filter::Running => a.status.eq_ignore_ascii_case("running"),
                        Filter::Unhealthy => a.status.eq_ignore_ascii_case("unhealthy"),
                        Filter::Favorites => false,
                    })
                    .collect();
            }
            Msg::Open(id) => {
                post_action(&format!("v1/apps/{id}/open"));
                let (connected, apps) = fetch_apps();
                self.connected = connected;
                self.apps = apps;
            }
            Msg::Stop(id) => {
                post_action(&format!("v1/apps/{id}/stop"));
                let (connected, apps) = fetch_apps();
                self.connected = connected;
                self.apps = apps;
            }
            Msg::Kill(id) => {
                post_action(&format!("v1/apps/{id}/kill"));
                let (connected, apps) = fetch_apps();
                self.connected = connected;
                self.apps = apps;
            }
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    let app = RelmApp::new("io.piercast.linux");
    app.run::<App>(());
}
