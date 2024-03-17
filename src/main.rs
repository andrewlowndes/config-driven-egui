#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, Ui};
use serde::{Deserialize, Serialize};
use std::{fs::File, ops::RangeInclusive};

// demo context
#[derive(Debug, Clone)]
pub struct HandlerContext {
    name: String,
    age: u32,
}

impl Default for HandlerContext {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

// demo handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerU32 {
    SetAge,
}

impl HandlerU32 {
    pub fn run(&self, value: u32, context: &mut HandlerContext) {
        match self {
            Self::SetAge => {
                context.age = value;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerString {
    SetName,
}

impl HandlerString {
    pub fn run(&self, value: String, context: &mut HandlerContext) {
        match self {
            Self::SetName => {
                context.name = value;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Handler {
    IncrementAge,
}

impl Handler {
    pub fn run(&self, context: &mut HandlerContext) {
        match self {
            Self::IncrementAge => {
                context.age += 1;
            }
        }
    }
}

// demo readers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsU32 {
    GetAge,

    #[serde(untagged)]
    Literal(u32),
}

impl AsU32 {
    pub fn as_u32(&self, context: &HandlerContext) -> u32 {
        match self {
            Self::GetAge => context.age,
            Self::Literal(val) => *val,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsString {
    GetName,
    Hello,

    #[serde(untagged)]
    Literal(String),
}

impl AsString {
    pub fn as_string(&self, context: &HandlerContext) -> String {
        match self {
            Self::GetName => context.name.clone(),
            Self::Hello => format!("Hello '{}', age {}", context.name, context.age),
            Self::Literal(val) => val.clone(),
        }
    }
}

// generic engine code below

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    text: AsString,
    id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalLayout {
    widgets: Vec<Widget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slider {
    range: RangeInclusive<u32>,
    text: AsString,
    value: AsU32,
    on_change: HandlerU32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Button {
    text: AsString,
    on_click: Handler,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    src: AsString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    value: AsString,
    on_change: HandlerString,
    label_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Widget {
    Label(Label),
    HorizontalLayout(HorizontalLayout),
    TextEdit(TextEdit),
    Slider(Slider),
    Button(Button),
    Image(Image),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralPanel {
    widgets: Vec<Widget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Container {
    CentralPanel(CentralPanel),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    name: String,
    containers: Vec<Container>,
}

pub struct Engine {
    config: Config,
    context: HandlerContext,
}

impl HandlerContext {
    pub fn update_widget(&mut self, widget: &Widget, ui: &mut Ui) {
        match widget {
            Widget::Label(label) => {
                ui.add(egui::widgets::Label::new(label.text.as_string(self)));
            }
            Widget::HorizontalLayout(layout) => {
                ui.horizontal(|ui| {
                    for widget in &layout.widgets {
                        self.update_widget(widget, ui);
                    }
                });
            }
            Widget::Slider(slider) => {
                let mut value = slider.value.as_u32(self);
                let response = ui.add(
                    egui::widgets::Slider::new(&mut value, slider.range.clone())
                        .text(slider.text.as_string(self)),
                );

                if response.changed() {
                    slider.on_change.run(value, self);
                }
            }
            Widget::Button(button) => {
                let button_response =
                    ui.add(egui::widgets::Button::new(button.text.as_string(self)));

                if button_response.clicked() {
                    button.on_click.run(self);
                }
            }
            Widget::Image(image) => {
                ui.add(egui::Image::from_uri(format!(
                    "file://{}",
                    image.src.as_string(self)
                )));
            }
            Widget::TextEdit(text_edit) => {
                let mut value = text_edit.value.as_string(self);
                let response = ui.add(egui::widgets::TextEdit::singleline(&mut value));

                if response.changed() {
                    text_edit.on_change.run(value, self);
                }
            }
        }
    }

    pub fn update_container(
        &mut self,
        container: &Container,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        match container {
            Container::CentralPanel(panel) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    for widget in &panel.widgets {
                        self.update_widget(widget, ui);
                    }
                });
            }
        }
    }
}

impl eframe::App for Engine {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // here we just construct each egui component that has been specified in the state (fully dynamic)
        for container in &self.config.containers {
            self.context.update_container(container, ctx, frame);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //deserialize the app from config
    let config_file = File::open("config/app.yaml")?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    let context = HandlerContext::default();

    let engine = Engine { config, context };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        engine.config.name.clone().as_str(),
        options,
        Box::new(move |cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(engine)
        }),
    )?;

    Ok(())
}
