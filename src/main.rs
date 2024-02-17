use hyprland::data::{Clients, Monitors, Workspaces};
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::shared::HyprData;
use hyprland::Result;
use serde::Serialize;
use serde_json::json;
use std::env;
use std::sync::Arc;

const HELP: &str = "\
hyprland-activewindow: a multi monitor aware active hyprland window title reporter, designed to be used with eww.

USAGE:
  hyprland-activewindow MONITOR

FLAGS:
  -h, --help            Prints help information

ARGS:
  <MONITOR>             Monitor to report active window title on or _ to report on all monitors
                        Note: using _ will output in json format
";

#[derive(Serialize)]
struct MonitorCustom {
    pub name: String,
    pub title: String,
    pub initial_title: String,
}

struct WindowPrinter {
    mon: String,
}

impl WindowPrinter {
    pub(crate) fn new(mon: String) -> Self {
        Self { mon }
    }

    pub fn print(&self) {
        if self.mon == "_" {
            self.print_all();
        } else {
            self.print_single();
        }
    }

    fn print_single(&self) {
        let active_workspace_id = Monitors::get()
            .expect("unable to get monitors")
            .find(|m| m.name == self.mon.to_string())
            .unwrap()
            .active_workspace
            .id;
        let title = Workspaces::get()
            .expect("unable to get workspaces")
            .find(|w| w.id == active_workspace_id)
            .unwrap()
            .last_window_title;
        println!("{}", title);
    }

    fn print_all(&self) {
        let monitors = Monitors::get().expect("unable to get monitors");
        let mut out_monitors: Vec<MonitorCustom> = Vec::new();
        for monitor in monitors {
            let workspace = Workspaces::get()
                .expect("unable to get workspaces")
                .find(|w| w.id == monitor.active_workspace.id)
                .unwrap();
            let client = Clients::get()
                .expect("unable to get clients")
                .find(|c| c.address == workspace.last_window)
                .unwrap();
                //.last_window_title;title
            let mc: MonitorCustom = MonitorCustom {
                name: monitor.name,
                title: client.title,
                initial_title: client.initial_title,
            };
            out_monitors.push(mc);
        }
        println!("{}", json!(out_monitors).to_string());
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    //check args
    if args.len() != 2 || args[1].eq("-h") || args[1].eq("--help") {
        println!("{HELP}");
        std::process::exit(0);
    }
    let mon = args[1].to_string();
    let mon_object = Monitors::get()
        .expect("unable to get monitors")
        .find(|m| m.name == mon);
    if mon_object.is_none() && mon != "_" {
        println!("Unable to find monitor {mon}");
        std::process::exit(0);
    }

    let wp = Arc::new(WindowPrinter::new(mon));
    wp.print();

    // Create a event listener
    let mut event_listener = EventListener::new();
    let wp_clone = Arc::clone(&wp);
    event_listener.add_active_window_change_handler(move |_, _| {
        wp_clone.print();
    });
    let wp_clone = Arc::clone(&wp);
    event_listener.add_window_close_handler(move |_, _| {
        wp_clone.print();
    });
    let wp_clone = Arc::clone(&wp);
    event_listener.add_workspace_change_handler(move |_, _| {
        wp_clone.print();
    });
    let wp_clone = Arc::clone(&wp);
    event_listener.add_window_moved_handler(move |_, _| {
        wp_clone.print();
    });

    event_listener.start_listener()
}
