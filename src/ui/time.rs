use gtk::glib::ControlFlow;
use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};
use time::OffsetDateTime;
use time::macros::format_description;

pub struct TimeModel {
    pub current_time: String,
    pub current_date: String,

    timer_id: Option<gtk::glib::SourceId>,
}

fn now_local() -> OffsetDateTime {
    OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
}

fn format_time(now: OffsetDateTime) -> String {
    let fmt = format_description!("[hour]:[minute]");
    now.format(&fmt).unwrap_or_default()
}

fn format_date(now: OffsetDateTime) -> String {
    let fmt = format_description!("[day] [month repr:short]");
    now.format(&fmt).unwrap_or_default()
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

                connect_show => move |popover| {
                    let now = now_local();
                    if popover.child().is_none() {
                        let calendar = gtk::Calendar::new();
                        calendar.add_css_class("nord-calendar");
                        calendar.set_show_heading(true);
                        calendar.set_show_day_names(true);

                        let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
                        wrapper.set_margin_start(8);
                        wrapper.set_margin_end(8);
                        wrapper.set_margin_top(8);
                        wrapper.set_margin_bottom(8);
                        wrapper.append(&calendar);

                        popover.set_child(Some(&wrapper));
                    }

                    // Refresh selected day on every popover open so the calendar reflects "today".
                    if let Some(child) = popover.child()
                        && let Some(wrapper) = child.downcast_ref::<gtk::Box>()
                        && let Some(first) = wrapper.first_child()
                        && let Some(calendar) = first.downcast_ref::<gtk::Calendar>()
                    {
                        let date = gtk::glib::DateTime::from_local(
                            now.year(),
                            now.month() as i32,
                            now.day() as i32,
                            now.hour() as i32,
                            now.minute() as i32,
                            f64::from(now.second()),
                        );
                        if let Ok(date) = date {
                            calendar.select_day(&date);
                        }
                    }
                },
            }
        },
    }

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let _ = sender;
        let _ = root;
        let now = now_local();
        let model = Self {
            current_time: format_time(now),
            current_date: format_date(now),
            timer_id: None,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            TimeMsg::WindowShown => {
                let now = now_local();
                self.current_time = format_time(now);
                self.current_date = format_date(now);

                if self.timer_id.is_none() {
                    let sender_clone = sender.clone();

                    let id = gtk::glib::timeout_add_seconds_local(1, move || {
                        sender_clone.input(TimeMsg::Tick);
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
                let now = now_local();
                let new_time = format_time(now);

                if self.current_time != new_time {
                    self.current_time = new_time;
                    self.current_date = format_date(now);
                }
            }
        }
    }
}

impl Drop for TimeModel {
    fn drop(&mut self) {
        if let Some(id) = self.timer_id.take() {
            id.remove();
        }
    }
}
