use chrono::Local;
use gtk::glib::ControlFlow;
use gtk4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

pub struct TimeModel {
    pub current_time: String,
    pub current_date: String,

    timer_id: Option<gtk::glib::SourceId>,
}

#[derive(Debug)]
pub enum TimeMsg {
    Tick,
    WindowShown,
    WindowHidden,
}

#[relm4::component(pub)]
impl SimpleComponent for TimeModel {
    type Input = TimeMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::MenuButton {
            #[watch]
            set_label: &format!("󰃰  {}    󰥔  {}", model.current_date, model.current_time),

            set_halign: gtk::Align::Start,
            add_css_class: "status-pill",
            set_always_show_arrow: false,
            set_has_frame: false,
            set_direction: gtk::ArrowType::None,

            connect_map[sender] => move |_| {
                sender.input(TimeMsg::WindowShown);
            },
            connect_unmap[sender] => move |_| {
                sender.input(TimeMsg::WindowHidden);
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_position: gtk::PositionType::Top,
                set_autohide: true,
                set_has_arrow: false,
                add_css_class: "nord-popover",

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 8,

                    gtk::Calendar {
                        add_css_class: "nord-calendar",
                        set_show_heading: true,
                        set_show_day_names: true,
                    }
                }
            }
        },
    }

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = Self {
            current_time: Local::now().format("%H:%M").to_string(),
            current_date: Local::now().format("%d %b").to_string(),
            timer_id: None,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            TimeMsg::WindowShown => {
                self.current_time = Local::now().format("%H:%M").to_string();
                self.current_date = Local::now().format("%d %b").to_string();

                if self.timer_id.is_none() {
                    let sender_clone = sender.clone();

                    let id = gtk::glib::timeout_add_seconds_local(1, move || {
                        let _ = sender_clone.input(TimeMsg::Tick);
                        ControlFlow::Continue
                    });

                    self.timer_id = Some(id);
                }
            }
            TimeMsg::WindowHidden => {
                if let Some(id) = self.timer_id.take() {
                    id.remove();
                }
            }
            TimeMsg::Tick => {
                let now = Local::now();
                let new_time = now.format("%H:%M").to_string();

                if self.current_time != new_time {
                    self.current_time = new_time;
                    self.current_date = now.format("%d %b").to_string();
                }
            }
        }
    }
}
