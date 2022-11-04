use done_core::services::provider::TaskStatus;
use relm4::factory::{
	DynamicIndex, FactoryComponent, FactoryComponentSender, FactoryView,
};
use relm4::{
	gtk,
	gtk::prelude::{
		BoxExt, ButtonExt, CheckButtonExt, EditableExt, EntryBufferExtManual,
		EntryExt, ListBoxRowExt, OrientableExt, ToggleButtonExt, WidgetExt,
	},
	RelmWidgetExt,
};

use crate::widgets::components::content::ContentInput;
use done_core::services::provider::Task;

#[derive(Debug)]
pub enum TaskInput {
	SetCompleted(bool),
	Favorite(DynamicIndex),
	ModifyTitle(String),
}

#[derive(Debug)]
pub enum TaskOutput {
	Remove(DynamicIndex),
	UpdateTask(Option<DynamicIndex>, Task),
}

#[derive(Debug, Clone)]
pub struct TaskData {
	pub data: Task,
}

#[relm4::factory(pub)]
impl FactoryComponent for TaskData {
	type ParentInput = ContentInput;
	type ParentWidget = gtk::Box;
	type CommandOutput = ();
	type Input = TaskInput;
	type Output = TaskOutput;
	type Init = TaskData;
	type Widgets = TaskWidgets;

	view! {
		root = gtk::ListBoxRow {
			set_selectable: false,
			#[name = "container"]
			gtk::Box {
				append = &gtk::Box {
					set_orientation: gtk::Orientation::Horizontal,
					set_spacing: 5,
					set_margin_top: 10,
					set_margin_bottom: 10,
					set_margin_start: 10,
					set_margin_end: 10,
					#[name = "check_button"]
					gtk::CheckButton {
						set_active: self.data.status == 1,
						connect_toggled[sender] => move |checkbox| {
							sender.input(TaskInput::SetCompleted(checkbox.is_active()));
						}
					},
					gtk::Box {
						set_orientation: gtk::Orientation::Horizontal,
						set_spacing: 15,
						#[name = "entry"]
						gtk::Entry {
							add_css_class: "flat",
							add_css_class: "no-border",
							set_hexpand: true,
							set_text: &self.data.title,
							connect_activate[sender] => move |entry| {
								let buffer = entry.buffer();
								sender.input(TaskInput::ModifyTitle(buffer.text()));
							},
							// connect_insert_text[sender] => move |entry, _, _| {
							// 	let buffer = entry.buffer();
							// 	sender.input(TaskInput::ModifyTitle(buffer.text()));
							// },
							// connect_delete_text[sender] => move |entry, _, _| {
							// 	let buffer = entry.buffer();
							// 	sender.input(TaskInput::ModifyTitle(buffer.text()));
							// }
						},
						#[name = "favorite"]
						gtk::ToggleButton {
							add_css_class: "opaque",
							add_css_class: "circular",
							#[watch]
							set_class_active: ("favorite", self.data.favorite),
							set_icon_name: "star-filled-rounded-symbolic",
							connect_toggled[sender, index] => move |_| {
								sender.input(TaskInput::Favorite(index.clone()));
							}
						},
						#[name = "delete"]
						gtk::Button {
							add_css_class: "destructive-action",
							add_css_class: "circular",
							set_icon_name: "user-trash-full-symbolic",
							connect_clicked[sender, index] => move |_| {
								sender.output(TaskOutput::Remove(index.clone()))
							}
						}
					}
				}
			}
		}
	}

	fn init_model(
		params: Self::Init,
		_index: &DynamicIndex,
		_sender: FactoryComponentSender<Self>,
	) -> Self {
		params
	}

	fn init_widgets(
		&mut self,
		index: &DynamicIndex,
		root: &Self::Root,
		_returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
		sender: FactoryComponentSender<Self>,
	) -> Self::Widgets {
		let widgets = view_output!();
		widgets
	}

	fn update(
		&mut self,
		message: Self::Input,
		sender: FactoryComponentSender<Self>,
	) {
		match message {
			TaskInput::SetCompleted(completed) => {
				self.data.status = if completed {
					TaskStatus::Completed as i32
				} else {
					TaskStatus::NotStarted as i32
				};
				sender
					.output_sender()
					.send(TaskOutput::UpdateTask(None, self.data.clone()));
			},
			TaskInput::Favorite(index) => {
				self.data.favorite = !self.data.favorite;
				sender
					.output_sender()
					.send(TaskOutput::UpdateTask(Some(index), self.data.clone()));
			},
			TaskInput::ModifyTitle(title) => {
				self.data.title = title;
				sender
					.output_sender()
					.send(TaskOutput::UpdateTask(None, self.data.clone()));
			},
		}
	}

	fn output_to_parent_input(output: Self::Output) -> Option<ContentInput> {
		Some(match output {
			TaskOutput::Remove(index) => ContentInput::RemoveTask(index),
			TaskOutput::UpdateTask(index, task) => {
				ContentInput::UpdateTask(index, task)
			},
		})
	}
}
