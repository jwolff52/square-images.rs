use std::{collections::HashMap, io::Cursor};

use base64::{Engine as _, engine::general_purpose};
use gloo_console::{info, error};
use gloo_file::{File, callbacks::FileReader};
use image::{EncodableLayout, GenericImageView, ImageFormat, DynamicImage};

use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_bootstrap::{util::{include_cdn, include_cdn_js, Color}, component::Spinner};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

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

#[derive(Clone, Debug, Default, PartialEq, Properties)]
pub struct Props {
    converting: bool,
    loading: bool,
}

pub struct App {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
    tile: Option<FileDetails>,
    new_files: Vec<FileDetails>,
    props: Props,
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
            props: Props {
                converting: false,
                loading: true,
            },
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
                        self.props.loading = !self.readers.is_empty();
                    }
                    FileType::File => {
                        self.files.push(FileDetails {
                            name: name.clone(),
                            file_type: mime_type,
                            data,
                        });
                        self.readers.remove(&name);
                        self.props.loading = !self.readers.is_empty();
                    }
                    FileType::Processed => {
                        self.new_files.push(FileDetails {
                            name: name.clone(),
                            file_type: mime_type,
                            data,
                        });
                        self.readers.remove(&name);
                        self.props.converting = !self.readers.is_empty();
                    }
                }
                true
            }
            Msg::Files(files) => {
                self.props.loading = true;
                for file in files {
                    let name = file.name();
                    let file_type = file.raw_mime_type();

                    let task = {
                        let link = ctx.link().clone();
                        let name = name.clone();

                        gloo_file::callbacks::read_as_bytes(&file, move |res| {
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
                        self.props.loading = true;
                        let name = file.name();
                        let file_type = file.raw_mime_type();

                        let task = {
                            let link = ctx.link().clone();
                            let name = name.clone();

                            gloo_file::callbacks::read_as_bytes(&file, move |res| {
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
              self.props.converting = true;
                for file in files {
                    let name = file.name();
                    let file_type = file.raw_mime_type();
                    let task = {
                        let link = ctx.link().clone();
                        let name = name.clone();

                        gloo_file::callbacks::read_as_bytes(&file, move |res| {
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
        info!(format!("{:#?}", self.props));
        html! {
            <div id="wrapper">
                {include_cdn()}
                <p id="title">{"Convert your pictures"}</p>
                <div id="upload-boxes">
                    <label for="tile-upload">
                        <div class="my-container"
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
                            <h4>{"Upload Tile Images"}</h4>
                            <p>{"Drag and drop file here"}<br/>
                            {"or"}<br/>
                            {"Click to select file"}</p>
                        </div>
                    </label>
                    <div
                        class="my-container"
                        onclick={
                          if self.props.loading || self.props.converting {
                            Callback::noop()
                          } else {
                            let files = self.files.clone();
                            let tile = self.tile.clone();
                            let props = ctx.props().clone();
                            ctx.link().callback(move |_| {
                                Self::convert_files(files.clone(), tile.clone(), props.converting)
                            })
                          }
                        }
                    >
                        {Self::button_text(self.props.clone())}
                    </div>
                    <label for="file-upload">
                        <div class="my-container"
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
                            <h4>{"Upload Original Images"}</h4>
                            <p>{"Drag and drop files here"}<br/>
                            {"or"}<br/>
                            {"Click to select files"}</p>
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
                {include_cdn_js()}
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
            let tile = image::load_from_memory(&tile.data);
            match tile {
                Ok(tile) => {
                    info!("Tile loaded");
                    let tile = tile.resize(256, 256, image::imageops::FilterType::Nearest);
                    let tile = tile.to_rgba8();
                    let tile = image::DynamicImage::ImageRgba8(tile);
                    for file in files {
                      result.push(Self::convert(file, tile.clone()));
                    }
                    Msg::ConvertedFiles(result)
                }
                Err(e) => {
                    error!(format!("Error loading tile: {}", e));
                    Msg::NoOp
                }
            }
        } else {
            Msg::NoOp
        }
    }

    fn convert(file: FileDetails, tile: DynamicImage) -> gloo_file::File {
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
        gloo_file::File::new_with_options::<&[u8]>(&file.name, new_buffer.into_inner().as_bytes(), Some(&file.file_type), None)
    }

    fn button_text(props: Props) -> Html {
        if props.loading {
            html_nested!(
                <>
                    <Spinner style={Color::Primary} />
                    <p>{"Loading"}</p>
                </>
            )
          } else if props.converting {
            html_nested!(
                <>
                    <Spinner style={Color::Primary} />
                    <p>{"Converting"}</p>
                </>
            )
          } else {
            html_nested!(<p>{"Convert"}</p>)
          }
    }
}