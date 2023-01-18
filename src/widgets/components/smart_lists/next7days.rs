use std::collections::HashMap;

use proto_rust::Task;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use crate::application::plugin::Plugin;

pub struct Next7DaysModel {
	pub tasks: HashMap<Plugin, Vec<Task>>,
}

#[derive(Debug)]
pub enum Next7DaysInput {}

#[derive(Debug)]
pub enum Next7DaysOutput {}

#[relm4::component(pub)]
impl SimpleComponent for Next7DaysModel {
	type Input = Next7DaysInput;

	type Output = Next7DaysOutput;

	type Init = ();

	view! {
			#[root]
			gtk::Box {
								gtk::Label {
										set_text: "Next 7 Days"
								}
			}
	}

	fn init(
		_init: Self::Init,
		root: &Self::Root,
		_sender: relm4::ComponentSender<Self>,
	) -> relm4::ComponentParts<Self> {
		let model = Self {
			tasks: HashMap::new(),
		};
		let widgets = view_output!();
		ComponentParts { model, widgets }
	}

	fn update(
		&mut self,
		_message: Self::Input,
		_sender: relm4::ComponentSender<Self>,
	) {
	}
}