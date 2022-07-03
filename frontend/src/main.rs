use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <h1 class={classes!("text-red-500")}>{ "Hello world!" } </h1>
    }
}

fn main() {
    yew::start_app::<App>();
}