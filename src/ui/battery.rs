use gtk::glib::ControlFlow;
use gtk4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

use crate::core::battery::{get_battery_info, BatteryInfo, BatteryStatus};

pub struct BatteryModel {
    info: Option<BatteryInfo>,
    timer_id: Option<gtk::glib::SourceId>,
}

#[derive(Debug)]
pub enum BatteryMsg {
    Tick,
    WindowShown,
    WindowHidden,
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

                #[wrap(Some)]
                set_child = &gtk::Grid {
                    set_margin_all: 18,
                    set_row_spacing: 16,
                    set_column_spacing: 32,

                    attach[0, 0, 1, 1] = &gtk::Label {
                        set_label: "Status",
                        set_halign: gtk::Align::Start,
                        add_css_class: "table-key",
                    },
                    attach[1, 0, 1, 1] = &gtk::Label {
                        #[watch]
                        set_label: &match &model.info {
                            Some(info) => match info.status {
                                BatteryStatus::Charging => "Charging",
                                BatteryStatus::Discharging => "Discharging",
                                BatteryStatus::Full => "Full",
                                BatteryStatus::Unknown => "Unknown",
                            },
                            None => "N/A",
                        },
                        set_halign: gtk::Align::End,
                        set_hexpand: true,
                        add_css_class: "table-value",
                    },

                    attach[0, 1, 1, 1] = &gtk::Label {
                        set_label: "Charge",
                        set_halign: gtk::Align::Start,
                        add_css_class: "table-key",
                    },
                    attach[1, 1, 1, 1] = &gtk::Label {
                        #[watch]
                        set_label: &match &model.info {
                            Some(info) => format!("{}%", info.capacity),
                            None => "N/A".to_string(),
                        },
                        set_halign: gtk::Align::End,
                        add_css_class: "table-value",
                    },

                    attach[0, 2, 1, 1] = &gtk::Label {
                        #[watch]
                        set_label: &match &model.info {
                            Some(info) if info.status == BatteryStatus::Charging => "Power",
                            _ => "Consumption",
                        },
                        set_halign: gtk::Align::Start,
                        add_css_class: "table-key",
                    },
                    attach[1, 2, 1, 1] = &gtk::Label {
                        #[watch]
                        set_label: &match &model.info {
                            Some(info) => format!("{:.1} W", info.power_w),
                            None => "0.0 W".to_string(),
                        },
                        set_halign: gtk::Align::End,
                        add_css_class: "table-value",
                    },

                    attach[0, 3, 1, 1] = &gtk::Label {
                        #[watch]
                        set_label: &match &model.info {
                            Some(info) if info.status == BatteryStatus::Charging => "Until full",
                            _ => "Remaining",
                        },
                        set_halign: gtk::Align::Start,
                        add_css_class: "table-key",
                    },
                    attach[1, 3, 1, 1] = &gtk::Label {
                        #[watch]
                        set_label: &match &model.info {
                            Some(info) if info.time_remaining_min > 0.0 && info.time_remaining_min < 1440.0 => {
                                let h = (info.time_remaining_min / 60.0) as i32;
                                let m = (info.time_remaining_min % 60.0) as i32;
                                if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
                            },
                            _ => "—".to_string(),
                        },
                        set_halign: gtk::Align::End,
                        add_css_class: "table-value",
                    },
                }
            }
        },
    }

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = Self {
            info: get_battery_info(),
            timer_id: None,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            BatteryMsg::WindowShown => {
                self.info = get_battery_info();

                if self.timer_id.is_none() {
                    let sender_clone = sender.clone();
                    let id = gtk::glib::timeout_add_seconds_local(5, move || {
                        let _ = sender_clone.input(BatteryMsg::Tick);
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
                }
            }
        }
    }
}
