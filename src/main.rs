use rayon::iter::ParallelIterator;
use std::{fs::DirEntry, path::Path, rc::Rc, sync::Arc};

use iced::{
    Event,
    Length::Fill,
    Subscription, Task, event,
    keyboard::Modifiers,
    task,
    widget::{button, column, container, mouse_area, row, scrollable, text, text_input},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FileType {
    Dir,
    File,
    Unknown,
}

// TODO: map when reading directory
#[derive(Debug)]
struct File {
    file_type: FileType,
    name: Arc<str>,
    path: Arc<str>,
}

impl From<DirEntry> for File {
    fn from(value: DirEntry) -> Self {
        let file_type = get_ft(&value);
        let name: Arc<str> = value.file_name().to_string_lossy().as_ref().into();
        let path: Arc<str> = value.path().to_string_lossy().as_ref().into();

        Self {
            file_type,
            name,
            path,
        }
    }
}

fn get_ft(entry: &DirEntry) -> FileType {
    let Ok(ft) = entry.file_type() else {
        return FileType::Unknown;
    };
    if ft.is_dir() {
        FileType::Dir
    } else if ft.is_file() {
        FileType::File
    } else {
        FileType::Unknown
    }
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default)]
struct App {
    path: Arc<str>,
    entries: Vec<File>,
    modifiers: Modifiers,

    // TODO: Arc<Vec<Arc<str>>> or am i just a baboon
    items_to_copy: Vec<String>,

    anchor: Option<usize>,
    // TODO: Set?
    selected_entries: Vec<usize>,

    err: Option<String>,
}

#[derive(Clone, Debug)]
enum Message {
    GoTo(Arc<str>),
    Open(Arc<str>),
    Select(usize),
    GoBack,
    PathChanged(String),
    Event(Event),
}

// TODO: Handle file does not exist
// TODO: handle errors with a nice error toast/shitter

impl App {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    fn new(path: String) -> App {
        // TODO: Remove unwrap logic
        let mut entries: Vec<File> = std::fs::read_dir(&path)
            .unwrap()
            .flatten()
            .map(|item| item.into())
            .collect();

        entries.sort_by(|a, b| a.file_type.cmp(&b.file_type).then(a.name.cmp(&b.name)));

        Self {
            path: path.into(),
            entries,
            selected_entries: Vec::with_capacity(255),
            ..Default::default()
        }
    }

    fn goto(&mut self, path: Arc<str>) {
        if let Ok(dir) = std::fs::read_dir(&*path) {
            self.entries = dir.flatten().map(|item| item.into()).collect();
            self.entries
                .sort_by(|a, b| a.file_type.cmp(&b.file_type).then(a.name.cmp(&b.name)));
            self.path = path;
        } else {
            self.err = Some("Could not open directory".into()).selkfsjd
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        #[cfg(debug_assertions)]
        let t = std::time::Instant::now();
        eprintln!("Update: {:?}", message.clone());

        match message {
            Message::GoTo(path) => self.goto(path),
            Message::Open(path) => {
                //TODO: delete this dep?
                let _ = open::that_detached(&*path);
            }
            Message::PathChanged(path) => self.path = path.into(),
            Message::GoBack => {
                let p = Path::new(&*self.path);
                if let Some(parent) = p.parent() {
                    let path = parent.to_string_lossy().into_owned();
                    self.goto(path.into());
                }
            }
            Message::Select(index) => {
                if self.modifiers.shift() {
                    if let Some(anchor) = self.anchor {
                        if anchor > index + 1 {
                            self.selected_entries = (index..anchor + 1).collect()
                        } else {
                            self.selected_entries = (anchor..index + 1).collect()
                        }
                    }
                } else if self.modifiers.control() {
                    if self.selected_entries.contains(&index) {
                        self.selected_entries.retain(|idx| index != *idx);
                    } else {
                        self.selected_entries.push(index);
                    }
                } else {
                    self.anchor = Some(index);
                    self.selected_entries.clear();
                    self.selected_entries.push(index);
                }
            }

            Message::Event(event) => match event {
                Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                    self.modifiers = modifiers
                }
                _ => {}
            },
        }

        #[cfg(debug_assertions)]
        eprintln!("Update: {:?}", t.elapsed());

        task::Task::none()
    }

    fn view(&self) -> Element<'_> {
        #[cfg(debug_assertions)]
        let t = std::time::Instant::now();

        let files = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, e)| -> Element<'_> {
                let ft_label = match e.file_type {
                    FileType::Dir => "\u{ea83}",
                    FileType::File => "\u{f15b}",
                    FileType::Unknown => "\u{f128}",
                };
                let name: &str = &e.name;
                let path = e.path.clone();

                let is_selected = self.selected_entries.contains(&index);

                let row = container(row![text(ft_label).width(24), text(&*name)])
                    .width(Fill)
                    .padding([2, 4])
                    .style(move |theme: &iced::Theme| {
                        if is_selected {
                            iced::widget::container::Style {
                                background: Some(
                                    theme.extended_palette().primary.weak.color.into(),
                                ),
                                ..Default::default()
                            }
                        } else {
                            iced::widget::container::Style::default()
                        }
                    })
                    .padding(10);

                match e.file_type {
                    FileType::Dir => mouse_area(row)
                        .on_press(Message::Select(index))
                        .on_double_click(Message::GoTo(path))
                        .into(),
                    _ => mouse_area(row)
                        .on_press(Message::Select(index))
                        .on_double_click(Message::Open(path))
                        .into(),
                }
            });

        let result = column![
            row![
                button("Back").on_press(Message::GoBack),
                text_input("Path...", &self.path)
                    .on_input(Message::PathChanged)
                    .on_submit(Message::GoTo(self.path.clone())),
            ],
            scrollable(column(files).width(Fill).padding(20))
        ]
        .into();

        #[cfg(debug_assertions)]
        eprintln!("view: {:?}", t.elapsed());

        result
    }
}

fn boot() -> App {
    let mut default_dir = "/".to_owned();
    if let Ok(dir) = std::env::var("HOME") {
        default_dir = dir
    }

    App::new(default_dir)
}

fn main() {
    iced::application(boot, App::update, App::view)
        .subscription(App::subscription)
        .run()
        .unwrap();
}
