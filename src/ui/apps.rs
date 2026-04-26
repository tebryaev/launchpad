use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

use crate::core::apps::{AppInfo, get_all_apps, launch_app};
use crate::core::usage::{load_usage, record_launch};

pub const MAX_APPLICATIONS: usize = 5;
const APP_TILE_HEIGHT: i32 = 64;

pub struct AppsModel {
    pub filter_query: String,
    pub all_apps: Vec<AppInfo>,
    pub displayed_apps: Vec<AppInfo>,
    pub widget_slots: Vec<AppWidgetSlot>,
    pub selected_idx: usize,
}

pub struct AppWidgetSlot {
    pub flow_child: gtk::FlowBoxChild,
    pub item_box: gtk::Box,
    pub icon: gtk::Image,
    pub label: gtk::Label,
}

#[derive(Debug)]
pub enum AppsMsg {
    LoadApps,
    UpdateFilter(String),
    Launch(usize),
    LaunchSelected,
    MoveSelection(i32, i32),
}

#[relm4::component(pub)]
impl SimpleComponent for AppsModel {
    type Input = AppsMsg;
    type Output = usize;
    type Init = ();

    view! {
        #[root]
        #[name = "flowbox"]
        gtk::FlowBox {
            set_selection_mode: gtk::SelectionMode::None,
            set_focusable: false,
            set_halign: gtk::Align::Fill,
            set_homogeneous: true,
            set_valign: gtk::Align::Start,
            set_max_children_per_line: MAX_APPLICATIONS as u32,
            set_min_children_per_line: 1,
            set_column_spacing: 8,
            set_row_spacing: 16,
            set_hexpand: true,
            set_vexpand: true,

            connect_child_activated[sender] => move |_, child| {
                sender.input(AppsMsg::Launch(child.index() as usize));
            }
        }
    }

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let widgets = view_output!();
        let mut widget_slots = Vec::with_capacity(MAX_APPLICATIONS);

        for _ in 0..MAX_APPLICATIONS {
            let item_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
            item_box.set_halign(gtk::Align::Fill);
            item_box.set_valign(gtk::Align::Start);
            item_box.set_size_request(-1, APP_TILE_HEIGHT);
            item_box.add_css_class("app-item-box");

            let icon = gtk::Image::builder()
                .pixel_size(APP_TILE_HEIGHT)
                .halign(gtk::Align::Center)
                .css_classes(["app-icon"])
                .build();

            let label = gtk::Label::builder()
                .css_classes(["app-label"])
                .halign(gtk::Align::Center)
                .justify(gtk::Justification::Center)
                .wrap(true)
                .wrap_mode(gtk::pango::WrapMode::WordChar)
                .lines(1)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .max_width_chars(10)
                .build();

            item_box.append(&icon);
            item_box.append(&label);

            let flow_child = gtk::FlowBoxChild::new();
            flow_child.set_focusable(false);
            // Keep slots visible from the start to reserve layout space —
            // empty-state styling is applied via the `.empty` CSS class.
            flow_child.set_visible(true);
            flow_child.set_can_target(false);
            flow_child.set_child(Some(&item_box));
            item_box.add_css_class("empty");

            widgets.flowbox.append(&flow_child);

            widget_slots.push(AppWidgetSlot {
                flow_child,
                item_box,
                icon,
                label,
            });
        }

        let model = Self {
            filter_query: String::new(),
            all_apps: Vec::new(),
            displayed_apps: Vec::new(),
            widget_slots,
            selected_idx: 0,
        };

        let load_sender = sender.clone();
        gtk::glib::idle_add_local_once(move || {
            load_sender.input(AppsMsg::LoadApps);
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppsMsg::LoadApps => {
                let mut apps = get_all_apps();
                let usage = load_usage();

                apps.sort_by(|a, b| {
                    let count_a = usage.get(&a.name).copied().unwrap_or(0);
                    let count_b = usage.get(&b.name).copied().unwrap_or(0);
                    count_b.cmp(&count_a).then_with(|| a.name.cmp(&b.name))
                });

                self.all_apps = apps;
                self.apply_filter(&sender);
            }
            AppsMsg::UpdateFilter(q) => {
                self.filter_query = q.to_lowercase();
                self.apply_filter(&sender);
            }
            AppsMsg::MoveSelection(dx, dy) => {
                let visible_count = self.displayed_apps.len();
                if visible_count == 0 {
                    return;
                }

                let new_idx = self.selected_idx as i32 + dx + (dy * MAX_APPLICATIONS as i32);
                self.selected_idx =
                    new_idx.clamp(0, visible_count.saturating_sub(1) as i32) as usize;

                self.update_selection_visuals();
            }
            AppsMsg::Launch(index) => {
                if let Some(app) = self.displayed_apps.get(index) {
                    record_launch(&app.name);
                    launch_app(&app.exec);
                    relm4::main_application().quit();
                }
            }
            AppsMsg::LaunchSelected => {
                if let Some(app) = self.displayed_apps.get(self.selected_idx) {
                    record_launch(&app.name);
                    launch_app(&app.exec);
                    relm4::main_application().quit();
                }
            }
        }
    }
}

impl AppsModel {
    fn apply_filter(&mut self, sender: &ComponentSender<Self>) {
        self.displayed_apps.clear();

        for app in &self.all_apps {
            if app.name.to_lowercase().contains(&self.filter_query) {
                self.displayed_apps.push(app.clone());
                if self.displayed_apps.len() == MAX_APPLICATIONS {
                    break;
                }
            }
        }

        self.selected_idx = 0;

        for (i, slot) in self.widget_slots.iter().enumerate() {
            if let Some(app) = self.displayed_apps.get(i) {
                slot.label.set_label(&app.name);

                if app.icon.starts_with('/') {
                    slot.icon.set_from_file(Some(&app.icon));
                } else {
                    slot.icon.set_icon_name(Some(&app.icon));
                }

                slot.item_box.remove_css_class("empty");
                slot.flow_child.set_can_target(true);
            } else {
                slot.label.set_label("");
                slot.icon.set_icon_name(None);
                slot.item_box.add_css_class("empty");
                slot.flow_child.set_can_target(false);
            }
        }

        self.update_selection_visuals();
        let _ = sender.output(self.displayed_apps.len());
    }

    fn update_selection_visuals(&self) {
        for (i, slot) in self.widget_slots.iter().enumerate() {
            if i >= self.displayed_apps.len() {
                slot.item_box.remove_css_class("selected-app");
                continue;
            }
            if i == self.selected_idx {
                slot.item_box.add_css_class("selected-app");
            } else {
                slot.item_box.remove_css_class("selected-app");
            }
        }
    }
}
