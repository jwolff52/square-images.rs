use std::{collections::HashMap, io::Cursor};

use base64::{Engine as _, engine::general_purpose};
use gloo::file::{File, callbacks::FileReader};
use gloo_console::{info};
use image::{EncodableLayout, GenericImageView, ImageFormat, DynamicImage};
use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::{Callback, Component, Context, html, TargetCast, Properties};

#[derive(Clone)]
struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

pub enum FileType {
    Tile,
    File,
    Processed,
}

pub enum Msg {
    Loaded(String, String, Vec<u8>, FileType),
    Files(Vec<File>),
    Tile(Option<File>),
    ConvertedFiles(Vec<File>),
    NoOp,
}

#[derive(Clone, Default, PartialEq, Properties)]
pub struct Props {
    converting: bool,
}

pub struct App {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
    tile: Option<FileDetails>,
    new_files: Vec<FileDetails>,
}

impl Component for App {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
            tile: None,
            new_files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(name, mime_type, data, file_type) => {
                match file_type {
                    FileType::Tile => {
                        self.tile = Some(FileDetails {
                            name: name.clone(),
                            file_type: mime_type,
                            data,
                        });
                        self.readers.remove(&name);
                    }
                    FileType::File => {
                        self.files.push(FileDetails {
                            name: name.clone(),
                            file_type: mime_type,
                            data,
                        });
                        self.readers.remove(&name);
                    }
                    FileType::Processed => {
                        self.new_files.push(FileDetails {
                            name: name.clone(),
                            file_type: mime_type,
                            data,
                        });
                        self.readers.remove(&name);
                    }
                }
                true
            }
            Msg::Files(files) => {
                for file in files {
                    let name = file.name();
                    let file_type = file.raw_mime_type();

                    let task = {
                        let link = ctx.link().clone();
                        let name = name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(name, file_type, res.expect("failed to read file"), FileType::File))
                        })
                    };

                    self.readers.insert(name, task);
                }
                true
            }
            Msg::Tile(file) => {
                match file {
                    Some(file) => {
                        let name = file.name();
                        let file_type = file.raw_mime_type();

                        let task = {
                            let link = ctx.link().clone();
                            let name = name.clone();

                            gloo::file::callbacks::read_as_bytes(&file, move |res| {
                                link.send_message(Msg::Loaded(name, file_type, res.expect("failed to read file"), FileType::Tile))
                            })
                        };

                        self.readers.insert(name, task);
                        true
                    }
                    None => {
                        true
                    }
                }
            }
            Msg::ConvertedFiles(files) => {
                for file in files {
                    let name = file.name();
                    let file_type = file.raw_mime_type();
                    let task = {
                        let link = ctx.link().clone();
                        let name = name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(name, file_type, res.expect("failed to read file"), FileType::Processed))
                        })
                    };
                    self.readers.insert(name, task);
                }
                true
            },
            Msg::NoOp => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        html! {
            <div id="wrapper" converting={true}>
                <p id="title">{"Convert your pictures"}</p>
                <div id="upload-boxes">
                    <label for="tile-upload">
                        <div id="tile-drop-container"
                            ondrop={ctx.link().callback(|event: DragEvent| {
                                event.prevent_default();
                                let files = event.data_transfer().unwrap().files();
                                Self::upload_tile(files)
                            })}
                            ondragover={Callback::from(|event: DragEvent| {
                                event.prevent_default();
                            })}
                            ondragenter={Callback::from(|event: DragEvent| {
                                event.prevent_default();
                            })}
                        >
                            <i class="fa fa-cloud-upload"></i>
                            <h4>{"Upload Tile Images"}</h4>
                            <p>{"Drag and drop file here"}</p>
                            <p>{"or"}</p>
                            <p>{"Click to select file"}</p>
                        </div>
                    </label>
                    <div
                        id="button-container"
                        onclick={
                            let files = self.files.clone();
                            let tile = self.tile.clone();
                            let props = ctx.props().clone();
                            ctx.link().callback(move |_| {
                                Self::convert_files(files.clone(), tile.clone(), props.converting)
                            })
                        }
                    >
                        {"Convert"}
                    </div>
                    <label for="file-upload">
                        <div id="drop-container"
                            ondrop={ctx.link().callback(|event: DragEvent| {
                                event.prevent_default();
                                let files = event.data_transfer().unwrap().files();
                                Self::upload_files(files)
                            })}
                            ondragover={Callback::from(|event: DragEvent| {
                                event.prevent_default();
                            })}
                            ondragenter={Callback::from(|event: DragEvent| {
                                event.prevent_default();
                            })}
                        >
                            <i class="fa fa-cloud-upload"></i>
                            <h4>{"Upload Original Images"}</h4>
                            <p>{"Drag and drop files here"}</p>
                            <p>{"or"}</p>
                            <p>{"Click to select files"}</p>
                        </div>
                    </label>
                </div>
                <input
                    id="tile-upload"
                    type="file"
                    accept="image/*"
                    multiple={false}
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::upload_tile(input.files())
                    })}
                />
                <input
                    id="file-upload"
                    type="file"
                    accept="image/*"
                    multiple={true}
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::upload_files(input.files())
                    })}
                />
                <div id="preview-area">
                    if let Some(tile) = &self.tile {
                        { Self::view_file(&tile, FileType::Tile) }
                    }
                    { for self.files.iter().map(|f| Self::view_file(f, FileType::File)) }
                    { for self.new_files.iter().map(|f| Self::view_file(f, FileType::Processed))}
                </div>
            </div>
        }
    }
}

impl App {
    fn view_file(file: &FileDetails, file_type: FileType) -> yew::Html {
        let base64 = general_purpose::STANDARD_NO_PAD.encode(&file.data);
        html! {
            <div class="preview-title">
                <p class="preview-name">{ format!("{}: {}", match file_type {
                    FileType::Tile => "Tile",
                    FileType::Processed => "Processed",
                    FileType::File => "Original",
                }, file.name) }</p>
                <div class="preview-media">
                    if file.file_type.contains("image") {
                        <img src={ format!("data:{};base64,{}", file.file_type, &base64) } />
                    } else {
                        <p>{"Unsupported file type"}</p>
                    }
                </div>
            </div>
        }
    }

    fn upload_files(files: Option<FileList>) -> Msg {
        info!("Upload files");
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        
        Msg::Files(result)
    }

    fn upload_tile(files: Option<FileList>) -> Msg {
        info!("Upload tile");
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        let result = result.get(0);

        match result {
            Some(file) => {
                Msg::Tile(Some(file.clone()))
            }
            None => {
                Msg::Tile(None)
            }
        }
    }

    fn convert_files(files: Vec<FileDetails>, tile: Option<FileDetails>, converting: bool) -> Msg {
        if converting {
            return Msg::NoOp;
        }
        info!("Convert");
        let mut result = Vec::new();
        if let Some(tile) = tile {
            info!("Loading tile");
            let tile = image::load_from_memory_with_format(&tile.data, ImageFormat::from_mime_type(&tile.file_type).unwrap()).unwrap();
            for file in files {
                result.push(Self::convert(file, tile.clone()));
            }
            Msg::ConvertedFiles(result)
        } else {
            Msg::NoOp
        }
    }

    fn convert(file: FileDetails, tile: DynamicImage) -> gloo::file::File {
        info!(format!("Loading file: {}", file.name));
        let old = image::load_from_memory_with_format(&file.data, ImageFormat::from_mime_type(&file.file_type).unwrap()).unwrap();
        let (width, height) = old.dimensions();
        info!(format!("{}x{}", width, height));
        let max = width.max(height);

        info!("Creating new image");
        let new = image::RgbaImage::new(max, max);
        let mut new = image::DynamicImage::ImageRgba8(new);

        info!("Tiling image background");
        image::imageops::tile(&mut new, &tile);

        info!("Overlaying old image");
        image::imageops::overlay(&mut new, &old, ((max - width) / 2) as i64, ((max - height) / 2) as i64);

        info!("Saving new image to buffer");
        let mut new_buffer = Cursor::new(vec![]);
        new.write_to(&mut new_buffer, image::ImageOutputFormat::Png).unwrap();
        
        info!("Pushing new file to result");
        gloo::file::File::new_with_options::<&[u8]>(&file.name, new_buffer.into_inner().as_bytes(), Some(&file.file_type), None)
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}