use web_sys::{Event, File, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FileUploadProps {
    pub on_file_selected: Callback<File>,
}

#[function_component(FileUpload)]
pub fn file_upload(props: &FileUploadProps) -> Html {
    let on_change = {
        let on_file_selected = props.on_file_selected.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    on_file_selected.emit(file);
                }
            }
        })
    };

    html! {
        <div class="file-upload">
            <h3>{"Upload Audio File"}</h3>
            <div class="upload-container">
                <input
                    type="file"
                    id="audio-file"
                    accept="audio/*"
                    onchange={on_change}
                />
                <label for="audio-file">
                    <span class="upload-icon">{"📂"}</span>
                    <span class="upload-text">{"Choose a file or drag it here"}</span>
                </label>
                <p class="file-info">{"Supported formats: WAV, MP3, FLAC, OGG"}</p>
            </div>
        </div>
    }
}
