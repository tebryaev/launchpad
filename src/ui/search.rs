use crate::ui::apps::{AppsModel, MAX_APPLICATIONS};
use crate::ui::text::TextModel;
use gtk4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, Controller, SimpleComponent};
use relm4::{Component, ComponentController};

pub struct SearchModel {
    search_query: String,
    app_count: usize,
    apps_component: Controller<AppsModel>,
    text_component: Controller<TextModel>,

    key_controller: gtk::EventControllerKey,
}

#[derive(Debug)]
pub enum SearchMsg {
    UpdateQuery(String),
    Submit,
    MoveSelection(i32, i32),
    UpdateAppCount(usize),
}

#[relm4::component(pub)]
impl SimpleComponent for SearchModel {
    type Input = SearchMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[name = "search_input"]
            gtk::Entry {
                set_placeholder_text: Some("Search..."),
                add_css_class: "spotlight-input",
                set_hexpand: true,
                set_has_frame: false,
                set_margin_top: 4,
                set_margin_bottom: 12,

                connect_map => move |entry| {
                    entry.grab_focus();
                },

                connect_changed[sender] => move |entry| {
                    sender.input(SearchMsg::UpdateQuery(entry.text().to_string()));
                },

                connect_activate[sender] => move |_| {
                    sender.input(SearchMsg::Submit);
                },

            },

            #[name = "main_stack"]
            gtk::Stack {
                set_hhomogeneous: true,

                add_named[Some("apps")] = model.apps_component.widget(),
                add_named[Some("calculator")] = model.text_component.widget(),

                #[watch]
                set_visible_child_name: if model.app_count == 0 {
                    "calculator"
                } else {
                    "apps"
                },
            }
        }
    }

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let apps_component = AppsModel::builder()
            .launch(())
            .forward(sender.input_sender(), SearchMsg::UpdateAppCount);

        let text_component = TextModel::builder().launch(()).detach();

        let key_controller = gtk::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);

        let sender_clone = sender.clone();
        key_controller.connect_key_pressed(move |_, key, _, modifiers| {
            use gtk::gdk::Key;
            let is_shift = modifiers.contains(gtk::gdk::ModifierType::SHIFT_MASK);

            match key {
                Key::Up => {
                    sender_clone.input(SearchMsg::MoveSelection(0, -1));
                    gtk::glib::Propagation::Stop
                }
                Key::Down => {
                    sender_clone.input(SearchMsg::MoveSelection(0, 1));
                    gtk::glib::Propagation::Stop
                }
                Key::Left => {
                    sender_clone.input(SearchMsg::MoveSelection(-1, 0));
                    gtk::glib::Propagation::Stop
                }
                Key::Right => {
                    sender_clone.input(SearchMsg::MoveSelection(1, 0));
                    gtk::glib::Propagation::Stop
                }
                Key::ISO_Left_Tab | Key::Tab if is_shift => {
                    sender_clone.input(SearchMsg::MoveSelection(-1, 0));
                    gtk::glib::Propagation::Stop
                }
                Key::Tab => {
                    sender_clone.input(SearchMsg::MoveSelection(1, 0));
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            }
        });

        let model = Self {
            search_query: "".to_string(),
            app_count: MAX_APPLICATIONS,
            apps_component,
            text_component,
            key_controller: key_controller.clone(),
        };

        let widgets = view_output!();

        widgets.search_input.add_controller(key_controller);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SearchMsg::UpdateQuery(query) => {
                self.search_query = query.clone();

                self.apps_component
                    .sender()
                    .send(crate::ui::apps::AppsMsg::UpdateFilter(query.clone()))
                    .unwrap();
                self.text_component
                    .sender()
                    .send(crate::ui::text::TextMsg::ProcessQuery(query))
                    .unwrap();
            }
            SearchMsg::Submit => {
                if self.app_count > 0 {
                    self.apps_component
                        .sender()
                        .send(crate::ui::apps::AppsMsg::LaunchSelected)
                        .unwrap();
                }
            }
            SearchMsg::MoveSelection(dx, dy) => {
                if self.app_count > 0 {
                    self.apps_component
                        .sender()
                        .send(crate::ui::apps::AppsMsg::MoveSelection(dx, dy))
                        .unwrap();
                }
            }
            SearchMsg::UpdateAppCount(count) => {
                self.app_count = count;

                if self.app_count == 0 {
                    self.key_controller
                        .set_propagation_phase(gtk::PropagationPhase::None);
                } else {
                    self.key_controller
                        .set_propagation_phase(gtk::PropagationPhase::Capture);
                }
            }
        }
    }
}
