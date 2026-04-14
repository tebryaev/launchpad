use crate::core::calculator::evaluate;
use gtk4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

pub struct TextModel {
    pub expression: String,
    pub result: String,
}

#[derive(Debug)]
pub enum TextMsg {
    ProcessQuery(String),
}

#[relm4::component(pub)]
impl SimpleComponent for TextModel {
    type Input = TextMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_vexpand: true,
            set_spacing: 12,

            gtk::Label {
                add_css_class: "calc-result",
                set_halign: gtk::Align::Center,
                set_selectable: true,

                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::WordChar,
                set_lines: 3,
                set_ellipsize: gtk::pango::EllipsizeMode::End,

                #[watch]
                set_label: if model.result.is_empty() {
                    "..."
                } else {
                    &model.result
                },
            },

            gtk::Label {
                add_css_class: "calc-expression",
                set_halign: gtk::Align::Center,

                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::WordChar,
                set_lines: 2,
                set_ellipsize: gtk::pango::EllipsizeMode::End,

                #[watch]
                set_label: if model.expression.is_empty() {
                    "Enter expression"
                } else {
                    &model.expression
                },
            }
        }
    }

    fn init(_init: (), root: Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = Self {
            expression: "".to_string(),
            result: "".to_string(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            TextMsg::ProcessQuery(q) => {
                self.expression = q.clone();

                if !q.trim().is_empty() {
                    if let Some(res) = evaluate(&q) {
                        self.result = res;
                    } else {
                        self.result = "...".to_string();
                    }
                } else {
                    self.result = "".to_string();
                }
            }
        }
    }
}
