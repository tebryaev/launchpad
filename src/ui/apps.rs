use gtk4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use crate::core::apps::{get_all_apps, launch_app, AppInfo};
use crate::core::usage::{load_usage, record_launch};

pub const MAX_APPLICATIONS: usize = 5;

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
            set_max_children_per_line: 5,
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
        let mut apps = get_all_apps();
        let usage = load_usage();

        apps.sort_by(|a, b| {
            let count_a = usage.get(&a.name).unwrap_or(&0);
            let count_b = usage.get(&b.name).unwrap_or(&0);
            count_b.cmp(count_a).then(a.name.cmp(&b.name))
        });

        let widgets = view_output!();
        let mut widget_slots = Vec::with_capacity(MAX_APPLICATIONS);

        for _ in 0..MAX_APPLICATIONS {
            let item_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
            item_box.set_halign(gtk::Align::Fill);
            item_box.set_valign(gtk::Align::Start);
            item_box.set_size_request(-1, 64);
            item_box.add_css_class("app-item-box");

            let icon = gtk::Image::builder()
                .pixel_size(64)
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
            flow_child.set_child(Some(&item_box));
            flow_child.set_visible(false);

            widgets.flowbox.append(&flow_child);

            widget_slots.push(AppWidgetSlot {
                flow_child,
                item_box,
                icon,
                label,
            });
        }

        let mut model = Self {
            filter_query: "".to_string(),
            all_apps: apps,
            displayed_apps: Vec::new(),
            widget_slots,
            selected_idx: 0,
        };

        model.update(AppsMsg::UpdateFilter("".to_string()), sender.clone());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppsMsg::UpdateFilter(q) => {
                self.filter_query = q.to_lowercase();
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

                        slot.flow_child.set_visible(true);
                    } else {
                        slot.flow_child.set_visible(false);
                    }
                }

                self.update_selection_visuals();
                let _ = sender.output(self.displayed_apps.len());
            }
            AppsMsg::MoveSelection(dx, dy) => {
                let visible_count = self.displayed_apps.len();
                if visible_count == 0 {
                    return;
                }

                let new_idx = self.selected_idx as i32 + dx + (dy * 5);
                self.selected_idx = new_idx.clamp(0, visible_count.saturating_sub(1) as i32) as usize;

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
    fn update_selection_visuals(&self) {
        for (i, slot) in self.widget_slots.iter().enumerate() {
            if !slot.flow_child.is_visible() {
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