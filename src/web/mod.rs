use yew::prelude::*;
use crate::query::parsing::scanner::Scanner;


#[function_component(App)]
fn app() -> Html {
    let onclick = Callback::from(move |_| {
        let input = String::from("x = x + 1");
        let mut scanner = Scanner::new(&input);
        &scanner.scanTokens();
    });
    html! {
        <div>
            <h1>{ "Hello World" }</h1>
            <button {onclick}>{ "Click" }</button>
        </div>
    }
}

pub fn run() {
    yew::Renderer::<App>::new().render();
}
