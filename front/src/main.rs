mod app;
mod routes;
mod components;
mod pages;
mod services;
mod stores;
mod types;
mod utils;
mod i18n;
mod hooks;

use app::App;
use utils::i18n_helper;
use i18n::Language;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    
    // 初始化国际化，默认使用中文
    i18n_helper::init_i18n(Language::ZhCn);
    
    yew::Renderer::<App>::new().render();
}
