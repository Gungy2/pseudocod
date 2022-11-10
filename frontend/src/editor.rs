use std::io::{BufReader, Cursor};
use yew::{html, Component, Context, Html, events::Event};
use web_sys::{EventTarget, HtmlTextAreaElement};
use wasm_bindgen::JsCast;
use interpreter::interpret;

pub enum Msg {
    Execute,
    EditorChange(String),
}

pub struct Editor {
    code: String,
    result: String,
}

impl Component for Editor {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            code: "".to_owned(),
            result: "".to_owned(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Execute => {
                let mut input = Cursor::new(vec![]);
                let mut output_stream: Vec<u8> = Vec::new();
                let mut out = Cursor::new(&mut output_stream);
                interpret(&mut input, &mut out, &self.code).expect("Wrong code!");
                self.result = String::from_utf8(output_stream).unwrap();
            }
            Msg::EditorChange(code) => {
                self.code = code;
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Msg::Execute);
        let onchange = ctx.link().callback(|event: Event| {
            let target: Option<EventTarget> = event.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok());
            input.map(|input| Msg::EditorChange(input.value())).unwrap()
        });
        html! {
            <>
                <textarea rows="100" class="text-area" id="editor" {onchange}/>
                <button id="execute" {onclick}>
                    <i id="play-arrow" class="material-symbols-rounded">{ "play_arrow" }</i>
                    { "Executa" }
                </button>
                <div id="console" class="text-area" >{ &self.result }</div>
            </>
        }
    }
}