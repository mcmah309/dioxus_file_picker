use std::ops::Deref;

use dioxus::prelude::*;

#[component]
pub(crate) fn Overlay(children: Element) -> Element {
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 1000;",
            {children}
        }
    }
}
