use gtk4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

use crate::core::notifications::{self, NotificationStatus};

pub struct NotificationModel {
    pub status: NotificationStatus,
}

#[derive(Debug)]
pub enum NotificationMsg {
    RefreshStatus,
    SetEnabled,
    SetMuted,
    ClearAll,
}

#[relm4::component(pub)]
impl SimpleComponent for NotificationModel {
    type Input = NotificationMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::MenuButton {
            set_label: "󰂚",
            add_css_class: "status-pill",
            set_always_show_arrow: false,
            set_has_frame: false,
            set_direction: gtk::ArrowType::None,

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                add_css_class: "nord-popover",
                set_has_arrow: false,
                set_position: gtk::PositionType::Top,

                connect_show[sender] => move |_| {
                    sender.input(NotificationMsg::RefreshStatus);
                },

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,
                    set_margin_all: 4,
                    set_width_request: 180,

                    gtk::Button {
                        add_css_class: "flat",
                        add_css_class: "menu-item-btn",
                        connect_clicked => NotificationMsg::SetEnabled,
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_spacing: 8,
                            gtk::Label {
                                add_css_class: "menu-checkmark",
                                #[watch]
                                set_label: if model.status == NotificationStatus::Enabled { "✓" } else { " " },
                            },
                            gtk::Label { set_label: "Enabled" }
                        }
                    },

                    gtk::Button {
                        add_css_class: "flat",
                        add_css_class: "menu-item-btn",
                        connect_clicked => NotificationMsg::SetMuted,
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_spacing: 8,
                            gtk::Label {
                                add_css_class: "menu-checkmark",
                                #[watch]
                                set_label: if model.status == NotificationStatus::Muted { "✓" } else { " " },
                            },
                            gtk::Label { set_label: "Do Not Disturb" }
                        }
                    },

                    gtk::Separator {
                        set_margin_top: 4,
                        set_margin_bottom: 4,
                    },

                    gtk::Button {
                        add_css_class: "flat",
                        add_css_class: "menu-item-btn",
                        connect_clicked => NotificationMsg::ClearAll,
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_spacing: 8,
                            gtk::Label {
                                add_css_class: "menu-checkmark",
                                set_label: " ",
                            },
                            gtk::Label { set_label: "Clear All" }
                        }
                    }
                }
            }
        },
    }

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = Self {
            // Initial stub value, actual status will be fetched on menu open
            status: NotificationStatus::Enabled,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            NotificationMsg::RefreshStatus => {
                // Called automatically when popover opens
                self.status = notifications::get_status();
            }
            NotificationMsg::SetEnabled => {
                notifications::enable();
                self.status = NotificationStatus::Enabled;
            }
            NotificationMsg::SetMuted => {
                notifications::mute();
                self.status = NotificationStatus::Muted;
            }
            NotificationMsg::ClearAll => {
                notifications::clear_all();
            }
        }
    }
}
