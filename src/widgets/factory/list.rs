use relm4::adw::prelude::ActionRowExt;
use relm4::factory::r#async::traits::AsyncFactoryComponent;
use relm4::factory::{
	AsyncFactoryComponentSender, DynamicIndex, FactoryView,
};
use std::str::FromStr;

use crate::gtk::prelude::{
	ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, WidgetExt,
};
use crate::widgets::factory::provider::ProviderInput;
use done_core::plugins::Plugin;
use done_core::services::provider::List;

use crate::{adw, gtk};

#[derive(Debug)]
pub enum ListInput {
	Select,
	Delete(DynamicIndex),
	Rename(String),
	ChangeIcon(String),
}

#[derive(Debug)]
pub enum ListOutput {
	Select(List),
	DeleteTaskList(DynamicIndex),
	Forward,
}

#[derive(Debug, Clone)]
pub struct ListData {
	pub data: List,
}

#[relm4::factory(pub async)]
impl AsyncFactoryComponent for ListData {
	type ParentInput = ProviderInput;
	type ParentWidget = adw::ExpanderRow;
	type CommandOutput = ();
	type Input = ListInput;
	type Output = ListOutput;
	type Init = ListData;
	type Widgets = ListWidgets;

	view! {
		#[root]
		gtk::ListBoxRow {
			adw::ActionRow {
				add_prefix = &gtk::Entry {
					set_hexpand: false,
					add_css_class: "flat",
					add_css_class: "no-border",
					#[watch]
					set_text: self.data.name.as_str(),
					set_margin_top: 10,
					set_margin_bottom: 10,
					connect_activate[sender] => move |entry| {
						let buffer = entry.buffer();
						sender.input(ListInput::Rename(buffer.text()));
					},
					// This crashes the program.
					// connect_changed[sender] => move |entry| {
					// 	let buffer = entry.buffer();
					// 	sender.input(ListInput::Rename(buffer.text()));
					// }
				},
				add_prefix = &gtk::MenuButton {
					#[watch]
					set_label: self.data.icon.as_ref().unwrap().as_str(),
					set_css_classes: &["flat", "image-button"],
					set_valign: gtk::Align::Center,
					#[wrap(Some)]
					set_popover = &gtk::EmojiChooser{
						connect_emoji_picked[sender] => move |_, emoji| {
							sender.input(ListInput::ChangeIcon(emoji.to_string()))
						}
					}
				},
				add_suffix = &gtk::Label {
					set_halign: gtk::Align::End,
					set_css_classes: &["dim-label", "caption"],
					// #[watch]
					// set_text: self.count.to_string().as_str(),
				},
				add_suffix = &gtk::Button {
					set_icon_name: "user-trash-full-symbolic",
					set_css_classes: &["circular", "image-button", "destructive-action"],
					set_valign: gtk::Align::Center,
					connect_clicked[sender, index] => move |_| {
						sender.input(ListInput::Delete(index.clone()))
					}
				},
			},
			add_controller = &gtk::GestureClick {
				connect_pressed[sender] => move |_, _, _, _| {
					sender.input(ListInput::Select);
					sender.output(ListOutput::Forward)
				}
			}
		}
	}

	fn init_widgets(
		&mut self,
		index: &DynamicIndex,
		root: &Self::Root,
		_returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
		sender: AsyncFactoryComponentSender<Self>,
	) -> Self::Widgets {
		let widgets = view_output!();
		widgets
	}

	async fn init_model(
		params: Self::Init,
		_index: &DynamicIndex,
		_sender: AsyncFactoryComponentSender<Self>,
	) -> Self {
		params
	}

	async fn update(
		&mut self,
		message: Self::Input,
		sender: AsyncFactoryComponentSender<Self>,
	) {
		if let Ok(provider) = Plugin::from_str(&self.data.provider) {
			let mut service = provider.connect().await.unwrap();
			match message {
				ListInput::Rename(name) => {
					let mut list = self.data.clone();
					list.name = name.clone();
					let response = service
						.update_list(list)
						.await
						.unwrap()
						.into_inner();
					if response.successful {
						self.data.name = name;
					}
				},
				ListInput::Delete(index) => {
					let response = service
						.delete_list(self.clone().data.id)
						.await
						.unwrap()
						.into_inner();
					if response.successful {
						sender.output(ListOutput::DeleteTaskList(index))
					}
				},
				ListInput::ChangeIcon(icon) => {
					let mut list = self.data.clone();
					list.icon = Some(icon.clone());
					let response = service
						.update_list(list)
						.await
						.unwrap()
						.into_inner();
					if response.successful {
						self.data.icon = Some(icon);
					}
				},
				ListInput::Select => {
					sender.output(ListOutput::Select(self.data.clone()))
				},
			}
		} else if let ListInput::Select = message {
			sender.output(ListOutput::Select(self.data.clone()))
		}
	}

	fn output_to_parent_input(output: Self::Output) -> Option<Self::ParentInput> {
		match output {
			ListOutput::Select(list) => Some(ProviderInput::ListSelected(list)),
			ListOutput::DeleteTaskList(index) => {
				Some(ProviderInput::DeleteTaskList(index))
			},
			ListOutput::Forward => Some(ProviderInput::Forward(true)),
		}
	}
}
