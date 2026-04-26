use crate::ui::battery::BatteryModel;
use crate::ui::notifications::NotificationModel;
use crate::ui::search::SearchModel;
use crate::ui::time::TimeModel;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::{Component, ComponentController};
use relm4::{ComponentParts, ComponentSender, Controller, SimpleComponent, gtk};

const CONTAINER_WIDTH: i32 = 640;
const WINDOW_TOP_OFFSET: i32 = 200;
const STATUS_BAR_SPACING: i32 = 8;

pub struct ApplicationModel {
    search_component: Controller<SearchModel>,
    notifications_component: Controller<NotificationModel>,
    battery_component: Controller<BatteryModel>,
    time_component: Controller<TimeModel>,
}

#[derive(Debug)]
pub enum ApplicationMsg {
    Close,
}

#[relm4::component(pub)]
impl SimpleComponent for ApplicationModel {
    view! {
        #[root]
        gtk::ApplicationWindow {
            add_css_class: "spotlight-overlay",

            add_controller = gtk::EventControllerKey {
                set_propagation_phase: gtk::PropagationPhase::Capture,

                connect_key_pressed[sender] => move |_controller, keyval, _keycode, _state| {
                    if keyval == gtk::gdk::Key::Escape {
                        sender.input(ApplicationMsg::Close);
                        return gtk::glib::Propagation::Stop;
                    }
                    gtk::glib::Propagation::Proceed
                }
            },

            gtk::Overlay {
                // Splash screen.
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,

                    // Click outside the spotlight container to close.
                    add_controller = gtk::GestureClick {
                        connect_pressed[sender] => move |_, _, _, _| {
                            sender.input(ApplicationMsg::Close);
                        }
                    }
                },

                // Main spotlight container.
                add_overlay = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,
                    set_margin_top: WINDOW_TOP_OFFSET,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        add_css_class: "spotlight-container",
                        set_width_request: CONTAINER_WIDTH,

                        model.search_component.widget(),

                        gtk::CenterBox {
                            add_css_class: "status-bar",

                            #[wrap(Some)]
                            set_start_widget = &gtk::Box {
                                model.time_component.widget(),
                            },

                            #[wrap(Some)]
                            set_end_widget = &gtk::Box {
                                set_spacing: STATUS_BAR_SPACING,
                                set_halign: gtk::Align::End,

                                model.notifications_component.widget(),
                                model.battery_component.widget(),
                            }
                        },
                    }
                }
            }
        }
    }

    type Input = ApplicationMsg;
    type Output = ();
    type Init = ();

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_layer(Layer::Overlay);
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Bottom, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);
        root.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

        let search_component = SearchModel::builder().launch(()).detach();
        let notifications_component = NotificationModel::builder().launch(()).detach();
        let battery_component = BatteryModel::builder().launch(()).detach();
        let time_component = TimeModel::builder().launch(()).detach();

        let model = Self {
            search_component,
            notifications_component,
            battery_component,
            time_component,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ApplicationMsg::Close => {
                log::debug!("Close requested. Shutting down.");
                relm4::main_application().quit();
            }
        }
    }
}
