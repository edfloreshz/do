use crate::config::PROFILE;
use crate::fl;
use crate::widgets::components::content::{ContentInput, ContentModel};
use crate::widgets::components::sidebar::{SidebarModel, SidebarOutput};
use crate::widgets::modals::about::AboutDialog;
use crate::setup::main_app;
use done_core::plugins::Plugin;
use done_core::services::provider::List;
use gtk::prelude::*;
use relm4::{
	actions::{ActionGroupName, RelmAction, RelmActionGroup},
	adw,
	component::{AsyncComponent, AsyncComponentController},
	gtk, ComponentBuilder, ComponentController, ComponentParts, ComponentSender,
	Controller, SimpleComponent,
};
use relm4::component::AsyncController;

pub struct App {
	message: Option<AppMsg>,
	sidebar_controller: AsyncController<SidebarModel>,
	content_controller: AsyncController<ContentModel>,
	about_dialog: Option<Controller<AboutDialog>>,
	content_title: Option<String>,
	warning_revealed: bool,
}

impl App {
	pub fn new(
		sidebar: AsyncController<SidebarModel>,
		content: AsyncController<ContentModel>,
		about_dialog: Option<Controller<AboutDialog>>,
	) -> Self {
		Self {
			message: None,
			sidebar_controller: sidebar,
			content_controller: content,
			about_dialog,
			content_title: None,
			warning_revealed: true,
		}
	}
}

#[derive(Debug)]
pub enum AppMsg {
	ListSelected(List),
	ProviderSelected(Plugin),
	CloseWarning,
	Folded,
	Unfolded,
	Forward,
	Back,
	Quit,
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(
	PreferencesAction,
	WindowActionGroup,
	"preferences"
);
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
relm4::new_stateless_action!(QuitAction, WindowActionGroup, "quit");

#[relm4::component(pub)]
impl SimpleComponent for App {
	type Input = AppMsg;
	type Output = ();
	type Widgets = AppWidgets;
	type Init = ();

	menu! {
		primary_menu: {
			section! {
				"_Preferences" => PreferencesAction,
				"_Keyboard" => ShortcutsAction,
				"_About Done" => AboutAction,
				"_Quit" => QuitAction,
			}
		}
	}

	view! {
		#[root]
		main_window = adw::ApplicationWindow::new(&main_app()) {
			set_default_width: 800,
			set_default_height: 700,
			connect_close_request[sender] => move |_| {
				sender.input(AppMsg::Quit);
				gtk::Inhibit(true)
			},

			#[wrap(Some)]
			set_help_overlay: shortcuts = &gtk::Builder::from_resource(
					"/dev/edfloreshz/Done/ui/gtk/help-overlay.ui"
			).object::<gtk::ShortcutsWindow>("help_overlay").unwrap() -> gtk::ShortcutsWindow {
				set_transient_for: Some(&main_window),
				set_application: Some(&crate::setup::main_app()),
			},

			add_css_class?: if PROFILE == "Devel" {
				Some("devel")
			} else {
				None
			},

			add_controller = &gtk::GestureClick {
				connect_pressed[sender] => move |_, _, _, _| {
					sender.input(AppMsg::CloseWarning)
				}
			},

			#[name = "overlay"]
			gtk::Overlay {
				#[wrap(Some)]
				set_child: stack = &gtk::Stack {
					set_hexpand: true,
					set_vexpand: true,
					set_transition_duration: 250,
					set_transition_type: gtk::StackTransitionType::Crossfade,
					add_child = &gtk::Box {
						set_orientation: gtk::Orientation::Vertical,
						append: leaflet = &adw::Leaflet {
							set_can_navigate_back: true,
							append: sidebar = &gtk::Box {
								set_orientation: gtk::Orientation::Vertical,
								set_width_request: 280,
								#[name = "sidebar_header"]
								adw::HeaderBar {
									set_show_end_title_buttons: false,
									set_title_widget: Some(&gtk::Label::new(Some("Done"))),
									pack_end = &gtk::MenuButton {
										set_icon_name: "open-menu-symbolic",
										set_menu_model: Some(&primary_menu),
									},
								},
								append: model.sidebar_controller.widget(),
							},
							append: &gtk::Separator::default(),
							append: content = &gtk::Box {
								set_orientation: gtk::Orientation::Vertical,
								#[name = "content_header"]
								append = &adw::HeaderBar {
									set_hexpand: true,
									set_show_start_title_buttons: true,
									#[watch]
									set_title_widget: Some(&gtk::Label::new(model.content_title.as_ref().map(|x| x.as_str()))),
									pack_start: go_back_button = &gtk::Button {
										set_icon_name: "go-previous-symbolic",
										set_visible: false,
										connect_clicked[sender] => move |_| {
											sender.input(AppMsg::Back);
										}
									}
								},
								append: model.content_controller.widget()
							},
							connect_folded_notify[sender] => move |leaflet| {
								if leaflet.is_folded() {
									sender.input(AppMsg::Folded);
								} else {
									sender.input(AppMsg::Unfolded);
								}
							}
						},
						append = &gtk::InfoBar {
							set_message_type: gtk::MessageType::Warning,
							#[watch]
							set_revealed: model.warning_revealed,
							gtk::Label {
								set_wrap: true,
								add_css_class: "warning",
								set_text: fl!("alpha-warning")
							}
						},
					}
				}
			}
		}
	}

	fn post_view() {
		if let Some(msg) = &model.message {
			match msg {
				AppMsg::Folded => {
					if model.content_title.is_some() {
						leaflet.set_visible_child(content);
					} else {
						leaflet.set_visible_child(sidebar);
					}
					go_back_button.set_visible(true);
					sidebar_header.set_show_start_title_buttons(true);
					sidebar_header.set_show_end_title_buttons(true);
				},
				AppMsg::Unfolded => {
					go_back_button.set_visible(false);
					sidebar_header.set_show_start_title_buttons(false);
					sidebar_header.set_show_end_title_buttons(false);
				},
				AppMsg::Forward => leaflet.set_visible_child(content),
				AppMsg::Back => leaflet.set_visible_child(sidebar),
				_ => {},
			}
		}
	}

	fn init(
		_init: Self::Init,
		root: &Self::Root,
		sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let actions = RelmActionGroup::<WindowActionGroup>::new();
		let mut model = App::new(
			SidebarModel::builder().launch(()).forward(
				sender.input_sender(),
				|message| match message {
					SidebarOutput::ListSelected(list) => AppMsg::ListSelected(list),
					SidebarOutput::Forward => AppMsg::Forward,
					SidebarOutput::ProviderSelected(plugin) => {
						AppMsg::ProviderSelected(plugin)
					},
				},
			),
			ContentModel::builder()
				.launch(None)
				.forward(sender.input_sender(), |message| match message {}),
			None,
		);

		let widgets = view_output!();

		let shortcuts_action = {
			let shortcuts = widgets.shortcuts.clone();
			RelmAction::<ShortcutsAction>::new_stateless(move |_| {
				shortcuts.present();
			})
		};

		let about_dialog = ComponentBuilder::default()
			.launch(widgets.main_window.upcast_ref::<gtk::Window>().clone())
			.detach();

		model.about_dialog = Some(about_dialog);

		let about_action = {
			let sender = model.about_dialog.as_ref().unwrap().sender().clone();
			RelmAction::<AboutAction>::new_stateless(move |_| {
				sender.send(());
			})
		};

		let quit_action = {
			RelmAction::<QuitAction>::new_stateless(move |_| {
				sender.input_sender().send(AppMsg::Quit)
			})
		};

		actions.add_action(&shortcuts_action);
		actions.add_action(&about_action);
		actions.add_action(&quit_action);

		widgets.main_window.insert_action_group(
			WindowActionGroup::NAME,
			Some(&actions.into_action_group()),
		);

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		match message {
			AppMsg::Quit => main_app().quit(),
			AppMsg::ListSelected(list) => {
				self.warning_revealed = false;
				self.content_title = Some(list.name.clone());
				self
					.content_controller
					.sender()
					.send(ContentInput::SetTaskList(list))
			},
			AppMsg::CloseWarning => self.warning_revealed = false,
			AppMsg::ProviderSelected(provider) => self
				.content_controller
				.sender()
				.send(ContentInput::SetProvider(provider)),
			_ => self.message = Some(message),
		}
	}
}
