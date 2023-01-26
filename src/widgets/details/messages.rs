use chrono::NaiveDateTime;
use proto_rust::Task;
use relm4::prelude::DynamicIndex;

use super::model::{DateDay, DateTpe};

#[derive(Debug)]
pub enum TaskDetailsFactoryInput {
	SaveTask,
	SetTitle(String),
	SetNotes(Option<String>),
	SetPriority(i32),
	SetFavorite(bool),
	SetStatus(bool),
	SetDueDate(Option<NaiveDateTime>),
	SetReminderDate(Option<NaiveDateTime>),
	CancelWarning,
	SetDate(DateTpe, DateDay),
}

#[derive(Debug)]
pub enum TaskDetailsFactoryOutput {
	SaveTask(Option<DynamicIndex>, Box<Task>, bool),
	CleanTaskEntry,
	HideFlap,
}
