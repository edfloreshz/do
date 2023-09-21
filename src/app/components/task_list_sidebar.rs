use core_done::{models::list::List, service::Service};
use futures::StreamExt;
use relm4::{
	adw,
	component::{AsyncComponentParts, SimpleAsyncComponent},
	factory::AsyncFactoryVecDeque,
	gtk::{
		self,
		prelude::BoxExt,
		traits::{ButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
	},
	prelude::DynamicIndex,
	tokio, AsyncComponentSender, Component, ComponentController, Controller,
	RelmWidgetExt,
};
use relm4_icons::icon_name;

use crate::{
	app::{
		components::list_dialog::ListDialogOutput,
		factories::task_list::{TaskListFactoryInit, TaskListFactoryModel},
		models::sidebar_list::SidebarList,
	},
	fl,
};

use super::list_dialog::ListDialogComponent;

pub struct TaskListSidebarModel {
	service: Service,
	state: TaskListSidebarStatus,
	task_list_factory: AsyncFactoryVecDeque<TaskListFactoryModel>,
	list_entry: Controller<ListDialogComponent>,
}

#[derive(Debug)]
pub enum TaskListSidebarInput {
	LoadTaskLists,
	OpenNewTaskListDialog,
	LoadTaskList(List),
	AddTaskListToSidebar(String),
	ServiceSelected(Service),
	ServiceDisabled(Service),
	SelectList(SidebarList),
	DeleteTaskList(DynamicIndex, String),
}

#[derive(Debug)]
pub enum TaskListSidebarOutput {
	SelectList(SidebarList, Service),
	CleanContent,
}

#[derive(Debug, PartialEq, Eq)]
enum TaskListSidebarStatus {
	Empty,
	Loading,
	Loaded,
}

#[relm4::component(pub async)]
impl SimpleAsyncComponent for TaskListSidebarModel {
	type Input = TaskListSidebarInput;
	type Output = TaskListSidebarOutput;
	type Init = Service;

	view! {
		#[root]
		adw::ToolbarView {
			add_top_bar = &adw::HeaderBar {
				set_css_classes: &["flat"],
				set_show_start_title_buttons: false,
				set_show_back_button: true,
				set_title_widget: Some(&gtk::Label::new(Some("Lists"))),
				pack_end = &gtk::Button {
					set_tooltip: fl!("add-new-task-list"),
					set_icon_name: icon_name::PLUS,
					set_css_classes: &["flat", "image-button"],
					set_valign: gtk::Align::Center,
					connect_clicked => TaskListSidebarInput::OpenNewTaskListDialog
				},
			},
			#[wrap(Some)]
			set_content = &gtk::Box {
				set_orientation: gtk::Orientation::Vertical,
				append = match model.state {
					TaskListSidebarStatus::Empty => {
						gtk::Box {
							set_margin_all: 20,
							set_vexpand: true,
							set_valign: gtk::Align::Center,
							set_css_classes: &["empty-state"],
							set_orientation: gtk::Orientation::Vertical,
							set_spacing: 10,
							gtk::Image {
								set_icon_name: Some(icon_name::DOCK_LEFT),
								set_pixel_size: 64,
								set_margin_all: 10,
							},
							gtk::Label {
								add_css_class: "title-2",
								set_label: fl!("empty-middle-tittle"),
							},
							gtk::Label {
								add_css_class: "body",
								set_label: fl!("middle-empty-instructions"),
							}
						}
					},
					TaskListSidebarStatus::Loading => {
						gtk::CenterBox {
							set_orientation: gtk::Orientation::Vertical,
							#[name(spinner)]
							#[wrap(Some)]
							set_center_widget = &gtk::Spinner {
								start: ()
							}
						}
					},
					TaskListSidebarStatus::Loaded => {
						gtk::ScrolledWindow {
							set_vexpand: true,
							#[local_ref]
							task_list_widget -> gtk::ListBox {
								set_css_classes: &["navigation-sidebar"],
							},
						}
					}
				}
			}
		}
	}

	async fn init(
		init: Self::Init,
		root: Self::Root,
		sender: AsyncComponentSender<Self>,
	) -> AsyncComponentParts<Self> {
		let model = TaskListSidebarModel {
			service: init,
			state: TaskListSidebarStatus::Empty,
			task_list_factory: AsyncFactoryVecDeque::new(
				gtk::ListBox::default(),
				sender.input_sender(),
			),
			list_entry: ListDialogComponent::builder().launch(None).forward(
				sender.input_sender(),
				|message| match message {
					ListDialogOutput::AddTaskListToSidebar(name) => {
						TaskListSidebarInput::AddTaskListToSidebar(name)
					},
					ListDialogOutput::RenameList(_) => todo!(),
				},
			),
		};
		sender.input(TaskListSidebarInput::LoadTaskLists);
		let task_list_widget = model.task_list_factory.widget();
		let widgets = view_output!();
		AsyncComponentParts { model, widgets }
	}

	async fn update(
		&mut self,
		message: Self::Input,
		sender: AsyncComponentSender<Self>,
	) {
		match message {
			TaskListSidebarInput::AddTaskListToSidebar(name) => {
				match self
					.service
					.get_service()
					.create_list(List::new(&name, self.service))
					.await
				{
					Ok(list) => {
						let mut guard = self.task_list_factory.guard();
						guard.push_back(TaskListFactoryInit::new(
							self.service,
							SidebarList::Custom(list),
						));
						self.state = TaskListSidebarStatus::Loaded;
					},
					Err(e) => {
						tracing::error!("Error while creating task list: {}", e);
					},
				}
			},
			TaskListSidebarInput::OpenNewTaskListDialog => {
				let list_entry = self.list_entry.widget();
				list_entry.present();
			},
			TaskListSidebarInput::ServiceSelected(service) => {
				self.service = service;
				self.state = TaskListSidebarStatus::Loading;
				sender.input(TaskListSidebarInput::LoadTaskLists);
			},
			TaskListSidebarInput::ServiceDisabled(service) => {
				if self.service == service {
					self.service = Service::Smart;
					self.state = TaskListSidebarStatus::Loading;
					sender.input(TaskListSidebarInput::LoadTaskLists);
				}
			},
			TaskListSidebarInput::LoadTaskList(list) => {
				let mut guard = self.task_list_factory.guard();
				guard.push_back(TaskListFactoryInit::new(
					self.service,
					SidebarList::Custom(list),
				));
				self.state = TaskListSidebarStatus::Loaded;
			},
			TaskListSidebarInput::LoadTaskLists => {
				let mut guard = self.task_list_factory.guard();
				guard.clear();

				let mut service = self.service.get_service();
				if service.stream_support() {
					let sender_clone = sender.clone();
					tokio::spawn(async move {
						let mut stream = service.get_lists().unwrap();
						while let Some(list) = stream.next().await {
							sender_clone.input(TaskListSidebarInput::LoadTaskList(list));
						}
					});
				} else {
					if matches!(self.service, Service::Smart) {
						for smart_list in SidebarList::list() {
							guard.push_back(TaskListFactoryInit::new(
								Service::Smart,
								smart_list,
							));
						}
					} else {
						for list in service.read_lists().await.unwrap() {
							guard.push_back(TaskListFactoryInit::new(
								self.service,
								SidebarList::Custom(list),
							));
						}
					}
					if guard.is_empty() {
						self.state = TaskListSidebarStatus::Empty;
					} else {
						self.state = TaskListSidebarStatus::Loaded;
					}
				}
			},
			TaskListSidebarInput::SelectList(list) => sender
				.output(TaskListSidebarOutput::SelectList(list, self.service))
				.unwrap(),
			TaskListSidebarInput::DeleteTaskList(index, list_id) => {
				match self.service.get_service().delete_list(list_id).await {
					Ok(_) => {
						self.task_list_factory.guard().remove(index.current_index());
						sender
							.output(TaskListSidebarOutput::CleanContent)
							.unwrap_or_default();
						if self.task_list_factory.is_empty() {
							self.state = TaskListSidebarStatus::Empty;
						}
					},
					Err(e) => {
						tracing::error!("Error while deleting task list: {}", e);
					},
				}
			},
		}
	}
}
