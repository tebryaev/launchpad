use gtk::glib::ControlFlow;
use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::core::battery::{
    BatteryInfo, BatteryStatus, get_battery_info, time_remaining_is_meaningful,
};

pub struct BatteryModel {
    info: Option<BatteryInfo>,
    timer_id: Option<gtk::glib::SourceId>,
    popover: Option<gtk::Popover>,
    popover_labels: Option<BatteryPopoverLabels>,
}

struct BatteryPopoverLabels {
    status: gtk::Label,
    charge: gtk::Label,
    power_key: gtk::Label,
    power: gtk::Label,
    remaining_key: gtk::Label,
    remaining: gtk::Label,
}

#[derive(Debug)]
pub enum BatteryMsg {
    Tick,
    WindowShown,
    WindowHidden,
    PopoverShow,
}

fn get_battery_icon(info: &BatteryInfo) -> &'static str {
    if info.status == BatteryStatus::Charging {
        return "󰂄";
    }

    match info.capacity {
        88..=100 => "󰁹",
        75..=87 => "󰂀",
        63..=74 => "󰁿",
        50..=62 => "󰁾",
        38..=49 => "󰁼",
        25..=37 => "󰁻",
        13..=24 => "󰁺",
        0..=12 => "󰂎",
        _ => "󰂃",
    }
}

fn status_text(info: &BatteryInfo) -> &'static str {
    match info.status {
        BatteryStatus::Charging => "Charging",
        BatteryStatus::Discharging => "Discharging",
        BatteryStatus::Full => "Full",
        BatteryStatus::Unknown => "Unknown",
    }
}

fn remaining_text(info: &BatteryInfo) -> String {
    match info.time_remaining_min {
        Some(min) if time_remaining_is_meaningful(min) => {
            let h = (min / 60.0) as i32;
            let m = (min % 60.0) as i32;
            if h > 0 {
                format!("{h}h {m}m")
            } else {
                format!("{m}m")
            }
        }
        _ => "—".to_string(),
    }
}

fn power_text(info: &BatteryInfo) -> String {
    match info.power_w {
        Some(w) => format!("{w:.1} W"),
        None => "N/A".to_string(),
    }
}

#[relm4::component(pub)]
impl SimpleComponent for BatteryModel {
    type Input = BatteryMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::MenuButton {
            #[watch]
            set_label: &match &model.info {
                Some(info) => format!("{}   {}%", get_battery_icon(info), info.capacity),
                None => "󰂑   N/A".to_string(),
            },

            add_css_class: "status-pill",
            set_always_show_arrow: false,
            set_has_frame: false,
            set_direction: gtk::ArrowType::None,

            connect_map[sender] => move |_| {
                sender.input(BatteryMsg::WindowShown);
            },
            connect_unmap[sender] => move |_| {
                sender.input(BatteryMsg::WindowHidden);
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                add_css_class: "nord-popover",
                set_position: gtk::PositionType::Top,
                set_has_arrow: false,

                connect_show[sender] => move |_| {
                    sender.input(BatteryMsg::PopoverShow);
                },
            }
        },
    }

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let _ = sender;
        // The `view!` macro references `model`, so we need an initial value before
        // `view_output!()`. After the widgets exist we attach the popover handle.
        let mut model = Self {
            info: None,
            timer_id: None,
            popover: None,
            popover_labels: None,
        };
        let widgets = view_output!();
        model.popover = root.popover();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            BatteryMsg::WindowShown => {
                self.info = get_battery_info();

                if self.timer_id.is_none() {
                    let sender_clone = sender.clone();
                    let id = gtk::glib::timeout_add_seconds_local(5, move || {
                        sender_clone.input(BatteryMsg::Tick);
                        ControlFlow::Continue
                    });
                    self.timer_id = Some(id);
                }
            }
            BatteryMsg::WindowHidden => {
                if let Some(id) = self.timer_id.take() {
                    id.remove();
                }
            }
            BatteryMsg::Tick => {
                let new_info = get_battery_info();
                if self.info != new_info {
                    self.info = new_info;
                    self.refresh_popover();
                }
            }
            BatteryMsg::PopoverShow => {
                self.info = get_battery_info();
                self.ensure_popover_content();
                self.refresh_popover();
            }
        }
    }
}

impl Drop for BatteryModel {
    fn drop(&mut self) {
        if let Some(id) = self.timer_id.take() {
            id.remove();
        }
    }
}

impl BatteryModel {
    fn ensure_popover_content(&mut self) {
        if self.popover_labels.is_some() {
            return;
        }
        let Some(popover) = &self.popover else { return };

        let grid = gtk::Grid::new();
        grid.set_margin_all(18);
        grid.set_row_spacing(16);
        grid.set_column_spacing(32);

        fn key_label(text: &str) -> gtk::Label {
            let l = gtk::Label::new(Some(text));
            l.set_halign(gtk::Align::Start);
            l.add_css_class("table-key");
            l
        }

        fn value_label() -> gtk::Label {
            let l = gtk::Label::new(None);
            l.set_halign(gtk::Align::End);
            l.add_css_class("table-value");
            l
        }

        let status_key = key_label("Status");
        let status = value_label();
        status.set_hexpand(true);

        let charge_key = key_label("Charge");
        let charge = value_label();

        let power_key = key_label("Consumption");
        let power = value_label();

        let remaining_key = key_label("Remaining");
        let remaining = value_label();

        grid.attach(&status_key, 0, 0, 1, 1);
        grid.attach(&status, 1, 0, 1, 1);
        grid.attach(&charge_key, 0, 1, 1, 1);
        grid.attach(&charge, 1, 1, 1, 1);
        grid.attach(&power_key, 0, 2, 1, 1);
        grid.attach(&power, 1, 2, 1, 1);
        grid.attach(&remaining_key, 0, 3, 1, 1);
        grid.attach(&remaining, 1, 3, 1, 1);

        popover.set_child(Some(&grid));

        self.popover_labels = Some(BatteryPopoverLabels {
            status,
            charge,
            power_key,
            power,
            remaining_key,
            remaining,
        });
    }

    fn refresh_popover(&self) {
        let Some(labels) = &self.popover_labels else {
            return;
        };
        match &self.info {
            Some(info) => {
                labels.status.set_label(status_text(info));
                labels.charge.set_label(&format!("{}%", info.capacity));
                let power_key = if info.status == BatteryStatus::Charging {
                    "Power"
                } else {
                    "Consumption"
                };
                labels.power_key.set_label(power_key);
                labels.power.set_label(&power_text(info));
                let remaining_key = if info.status == BatteryStatus::Charging {
                    "Until full"
                } else {
                    "Remaining"
                };
                labels.remaining_key.set_label(remaining_key);
                labels.remaining.set_label(&remaining_text(info));
            }
            None => {
                labels.status.set_label("N/A");
                labels.charge.set_label("N/A");
                labels.power_key.set_label("Consumption");
                labels.power.set_label("N/A");
                labels.remaining_key.set_label("Remaining");
                labels.remaining.set_label("—");
            }
        }
    }
}
